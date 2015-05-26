// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::rc::Rc;
use std::iter::FromIterator;
use syntax::ast;
use syntax::ptr::P;
use syntax::ast_util::empty_generics;
use syntax::codemap::{DUMMY_SP, dummy_spanned};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::parse::token;

use node;
use super::Builder;
use super::utils;

enum RegOrPadding<'a> {
  /// A register
  Reg(&'a node::Reg),
  /// A given number of bytes of padding
  Pad(u64)
}

/// An iterator which takes a potentially unsorted list of registers,
/// sorts them, and adds padding to make offsets correct
struct PaddedRegsIterator<'a> {
  sorted_regs: &'a Vec<node::Reg>,
  index: usize,
  last_offset: u64,
}

impl<'a> PaddedRegsIterator<'a> {
  fn new(regs: &'a mut Vec<node::Reg>) -> PaddedRegsIterator<'a> {
    regs.sort_by(|r1,r2| r1.offset.cmp(&r2.offset));
    PaddedRegsIterator {
      sorted_regs: regs,
      index: 0,
      last_offset: 0,
    }
  }
}

impl<'a> Iterator for PaddedRegsIterator<'a> {
  type Item = RegOrPadding<'a>;
  fn next(&mut self) -> Option<RegOrPadding<'a>> {
    if self.index >= self.sorted_regs.len() {
      None
    } else {
      let ref reg = self.sorted_regs[self.index];
      if reg.offset > self.last_offset {
        let pad_length = reg.offset - self.last_offset;
        self.last_offset = reg.offset;
        Some(RegOrPadding::Pad(pad_length))
      } else {
        self.index += 1;
        self.last_offset += reg.size();
        Some(RegOrPadding::Reg(reg))
      }
    }
  }
}

/// Build types for `RegUnions`
pub struct BuildUnionTypes<'a> {
  builder: &'a mut Builder,
  cx: &'a ExtCtxt<'a>
}

impl<'a> BuildUnionTypes<'a> {
  pub fn new(builder: &'a mut Builder, cx: &'a ExtCtxt<'a>)
      -> BuildUnionTypes<'a> {
    BuildUnionTypes { builder: builder, cx: cx }
  }
}

fn expr_u64(cx: &ExtCtxt, n: u64) -> P<ast::Expr> {
  cx.expr_lit(DUMMY_SP, ast::LitInt(n as u64, ast::UnsignedIntLit(ast::TyUs)))
}

/// Returns the type of the field representing the given register
/// within a `RegGroup` struct
fn reg_struct_type(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg)
                   -> P<ast::Ty> {
  let base_ty_path = cx.path_ident(DUMMY_SP, utils::path_ident(cx, path));
  let base_ty: P<ast::Ty> = cx.ty_path(base_ty_path);
  match reg.count.node {
    1 => base_ty,
    n =>
      cx.ty(DUMMY_SP,
            ast::TyFixedLengthVec(base_ty, expr_u64(cx, n as u64))),
  }
}


impl<'a> node::RegVisitor for BuildUnionTypes<'a> {
  fn visit_union_reg<'b>(&'b mut self, path: &Vec<String>, reg: &'b node::Reg,
                         subregs: Rc<Vec<node::Reg>>) {
    let items = self.build_union_type(path, reg, &*subregs);
    for item in items.into_iter() {
      self.builder.push_item(item);
    }
  }
}

impl<'a> BuildUnionTypes<'a> {
  /// Produce a field for the given register in a `RegUnion` struct
  fn build_reg_union_field(&self, path: &Vec<String>, reg: &node::Reg)
                           -> ast::StructField {
    let attrs = match reg.docstring {
      Some(doc) => vec!(utils::doc_attribute(self.cx, token::get_ident(doc.node))),
      None => Vec::new(),
    };
    let mut field_path = path.clone();
    field_path.push(reg.name.node.clone());
    dummy_spanned(
      ast::StructField_ {
        kind: ast::NamedField(
          self.cx.ident_of(reg.name.node.as_str()),
          ast::Public),
        id: ast::DUMMY_NODE_ID,
        ty: reg_struct_type(self.cx, &field_path, reg),
        attrs: attrs,
      }
    )
  }

  /// Build field for padding or a register
  fn build_pad_or_reg(&self, path: &Vec<String>, reg_or_pad: RegOrPadding,
                      index: usize) -> ast::StructField {
    match reg_or_pad {
      RegOrPadding::Reg(reg) => self.build_reg_union_field(path, reg),
      RegOrPadding::Pad(length) => {
        let u8_path = self.cx.path_ident(
          DUMMY_SP,
          self.cx.ident_of("u8"));
        let u8_ty: P<ast::Ty> = self.cx.ty_path(u8_path);
        let ty: P<ast::Ty> =
          self.cx.ty(
            DUMMY_SP,
            ast::TyFixedLengthVec(u8_ty, expr_u64(self.cx, length)));
        dummy_spanned(
          ast::StructField_ {
            kind: ast::NamedField(
              self.cx.ident_of(format!("_pad{}", index).as_str()),
              ast::Inherited),
            id: ast::DUMMY_NODE_ID,
            ty: ty,
            attrs: Vec::new(),
          },
        )
      },
    }
  }

  /// Build the type associated with a register group
  fn build_union_type(&self, path: &Vec<String>, reg: &node::Reg,
                      regs: &Vec<node::Reg>) -> Vec<P<ast::Item>> {
    let name = utils::path_ident(self.cx, path);
    // Registers are already sorted by parser
    let mut regs = regs.clone();
    let mut regs2 = regs.clone();
    let padded_regs = PaddedRegsIterator::new(&mut regs);
    let fields =
      padded_regs.enumerate().map(|(n,r)| self.build_pad_or_reg(path, r, n));
    let struct_def = ast::StructDef {
      fields: FromIterator::from_iter(fields),
      ctor_id: None,
    };
    let mut attrs: Vec<ast::Attribute> = vec!(
      utils::list_attribute(self.cx, "allow",
                            vec!("non_camel_case_types",
                                 "dead_code",
                                 "missing_docs")),
    );
    match reg.docstring {
      Some(docstring) =>
        attrs.push(
          utils::doc_attribute(self.cx, token::get_ident(docstring.node))),
      None => (),
    }
    let struct_item = P(ast::Item {
      ident: name,
      attrs: attrs,
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemStruct(P(struct_def), empty_generics()),
      vis: ast::Public,
      span: reg.name.span,
    });
    let mut full_size: u64 = 0;
    //FIXME(mcoffin) - We're making this iterator twice
    let padded_regs2 = PaddedRegsIterator::new(&mut regs2);
    padded_regs2.enumerate().map(|(_, rp)| {
      full_size += match rp {
        RegOrPadding::Reg(reg) => reg.ty.size(),
        RegOrPadding::Pad(s) => s,
      };
    }).count();
    let clone_impl = quote_item!(self.cx,
      impl ::core::clone::Clone for $name {
        fn clone(&self) -> Self {
          let mut next: $name = unsafe {
            ::core::mem::uninitialized()
          };
          unsafe {
            let next_ptr: *mut $name = &mut next;
            ::core::intrinsics::copy(
              ::core::mem::transmute(self),
              next_ptr,
              $full_size as usize);
            return next;
          }
        }
      }
    ).unwrap();
    let copy_impl = quote_item!(self.cx, impl ::core::marker::Copy for $name {}).unwrap();
    vec!(struct_item, clone_impl, copy_impl)
  }
}
