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

use std::gc::{Gc, GC};
use std::iter::FromIterator;
use syntax::ast;
use syntax::ast::P;
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
  Pad(uint)
}

/// An iterator which takes a potentially unsorted list of registers,
/// sorts them, and adds padding to make offsets correct
struct PaddedRegsIterator<'a> {
  sorted_regs: &'a Vec<node::Reg>,
  index: uint,
  last_offset: uint,
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

impl<'a> Iterator<RegOrPadding<'a>> for PaddedRegsIterator<'a> {
  fn next(&mut self) -> Option<RegOrPadding<'a>> {
    if self.index >= self.sorted_regs.len() {
      None
    } else {
      let ref reg = self.sorted_regs[self.index];
      if reg.offset > self.last_offset {
        let pad_length = reg.offset - self.last_offset;
        self.last_offset = reg.offset + reg.size();
        Some(Pad(pad_length))
      } else {
        self.index += 1;
        self.last_offset += reg.size();
        Some(Reg(reg))
      }
    }
  }
}

/// Build types for `RegUnions`
pub struct BuildUnionTypes<'a, 'b, 'c> {
  builder: &'a mut Builder,
  cx: &'b ExtCtxt<'c>
}

impl<'a, 'b, 'c> BuildUnionTypes<'a, 'b, 'c> {
  pub fn new(builder: &'a mut Builder,
             cx: &'b ExtCtxt<'c>)
             -> BuildUnionTypes<'a, 'b, 'c> {
    BuildUnionTypes { builder: builder, cx: cx }
  }
}

/// Returns the type of the field representing the given register
/// within a `RegGroup` struct
fn reg_struct_type(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg)
                   -> P<ast::Ty> {
  let base_ty_path = cx.path_ident(DUMMY_SP, utils::path_ident(cx, path));
  let base_ty: P<ast::Ty> = cx.ty_path(base_ty_path, None);
  match reg.count.node {
    1 => base_ty,
    n =>
      cx.ty(DUMMY_SP,
            ast::TyFixedLengthVec(base_ty,
                                  cx.expr_uint(DUMMY_SP, n))),
  }
}


impl<'a, 'b, 'c> node::RegVisitor for BuildUnionTypes<'a, 'b, 'c> {
  fn visit_union_reg<'a>(&'a mut self, path: &Vec<String>, reg: &'a node::Reg,
                         subregs: Gc<Vec<node::Reg>>) {
    let union_type = self.build_union_type(path, reg, &*subregs);
    self.builder.push_item(union_type);
  }
}

impl<'a, 'b, 'c> BuildUnionTypes<'a, 'b, 'c> {
  /// Produce a field for the given register in a `RegUnion` struct
  fn build_reg_union_field(&self, path: &Vec<String>, reg: &node::Reg)
                           -> ast::StructField {
    let attrs = match reg.docstring {
      Some(doc) => vec!(utils::doc_attribute(self.cx, token::get_ident(doc.node))),
      None => Vec::new(),
    };
    let field_path = path.clone().append_one(reg.name.node.clone());
    dummy_spanned(
      ast::StructField_ {
        kind: ast::NamedField(
          self.cx.ident_of(reg.name.node.as_slice()),
          ast::Public),
        id: ast::DUMMY_NODE_ID,
        ty: reg_struct_type(self.cx, &field_path, reg),
        attrs: attrs,
      }
    )
  }

  /// Build field for padding or a register
  fn build_pad_or_reg<'a>(&self, path: &Vec<String>, regOrPad: RegOrPadding<'a>,
                          index: uint) -> ast::StructField {
    match regOrPad {
      Reg(reg) => self.build_reg_union_field(path, reg),
      Pad(length) => {
        let u8_path = self.cx.path_ident(
          DUMMY_SP,
          self.cx.ident_of("u8"));
        let u8_ty: P<ast::Ty> = self.cx.ty_path(u8_path, None);
        let ty: P<ast::Ty> =
          self.cx.ty(
            DUMMY_SP,
            ast::TyFixedLengthVec(u8_ty,
                                  self.cx.expr_uint(DUMMY_SP, length)));
        dummy_spanned(
          ast::StructField_ {
            kind: ast::NamedField(
              self.cx.ident_of(format!("_pad{}", index).as_slice()),
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
                      regs: &Vec<node::Reg>) -> P<ast::Item> {
    let name = dummy_spanned(
      String::from_str(
        token::get_ident(utils::path_ident(self.cx, path)).get()));
    // Registers are already sorted by parser
    let mut regs = regs.clone();
    let padded_regs = PaddedRegsIterator::new(&mut regs);
    let fields =
      padded_regs.enumerate().map(|(n,r)| self.build_pad_or_reg(path, r, n));
    let struct_def = ast::StructDef {
      fields: FromIterator::from_iter(fields),
      ctor_id: None,
      super_struct: None,
      is_virtual: false,
    };
    let mut attrs: Vec<ast::Attribute> = vec!(
      utils::list_attribute(self.cx, "allow",
                            vec!("non_camel_case_types",
                                 "uppercase_variables",
                                 "dead_code",
                                 "missing_doc")),
    );
    match reg.docstring {
      Some(docstring) =>
        attrs.push(
          utils::doc_attribute(self.cx, token::get_ident(docstring.node))),
      None => (),
    }
    box(GC) ast::Item {
      ident: self.cx.ident_of(name.node.as_slice()),
      attrs: attrs,
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemStruct(box(GC) struct_def, empty_generics()),
      vis: ast::Public,
      span: name.span,
    }
  }
}
