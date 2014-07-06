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
use std::collections::hashmap::HashMap;
use std::iter::FromIterator;
use syntax::abi;
use syntax::ast::TokenTree;
use syntax::ast;
use syntax::ast::{P};
use syntax::ast_util::empty_generics;
use syntax::codemap::{Span, Spanned, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::owned_slice;
use syntax::parse::token;

use node;

pub struct Builder<'a, 'b> {
  groups: HashMap<String, Gc<node::RegGroup>>,
  cx: &'a mut ExtCtxt<'b>
}


impl<'a, 'b> Builder<'a, 'b> {
  pub fn new<'a, 'b>(cx: &'a mut ExtCtxt<'b>, groups: HashMap<String, Gc<node::RegGroup>>) -> Builder<'a, 'b> {
    Builder {
      groups: groups,
      cx: cx,
    }
  }

  /// Generate a `#[allow(...)]` attribute of the given type
  fn allow_attribute(&self, allow: &'static str) -> ast::Attribute {
    let word = self.cx.meta_word(DUMMY_SP, token::InternedString::new(allow));
    let allow = self.cx.meta_list(DUMMY_SP, token::InternedString::new("allow"), vec!(word));
    self.cx.attribute(DUMMY_SP, allow)
  }

  /// Generate a `#[doc="..."]` attribute of the given type
  fn doc_attribute(&self, docstring: token::InternedString) -> ast::Attribute {
    let s: ast::Lit_ = ast::LitStr(docstring, ast::CookedStr);
    let attr = self.cx.meta_name_value(DUMMY_SP, token::InternedString::new("doc"), s);
    self.cx.attribute(DUMMY_SP, attr)
  }

  pub fn emit_items(&self) -> Vec<Gc<ast::Item>> {
    let iter = self.groups.values().flat_map(|&g| self.emit_group(g).move_iter());
    FromIterator::from_iter(iter)
  }

  /// Returns the primitive type of the given width
  fn primitive_type(&self, width: uint) -> Option<P<ast::Ty>> {
    let name = match width {
      8  => "u8",
      16 => "u16",
      32 => "u32",
      _  => return None
    };
    let path = self.cx.path(DUMMY_SP, vec!(self.cx.ident_of("core"),
                                           self.cx.ident_of(name)));
    Some(self.cx.ty_path(path, None))
  }

  /// Produce a register struct if necessary (in the case of primitive typed registers).
  /// For instance,
  ///
  ///     pub struct REG {_value: u32}
  fn emit_reg_struct(&self, group: P<node::RegGroup>, reg: &node::Reg) -> Option<P<ast::Item>> {
    match reg.ty {
      node::GroupReg(_) => None,
      node::UIntReg(width) => {
        let ty = match self.primitive_type(width) {
          Some(ty) => ty,
          None => return None,
        };
        let struct_def = ast::StructDef {
          fields: vec!(
            Spanned {
              span: DUMMY_SP,
              node: ast::StructField_ {
                kind: ast::NamedField(self.cx.ident_of("_value"), ast::Inherited),
                id: ast::DUMMY_NODE_ID,
                ty: ty,
                attrs: Vec::new(),
              },
            },
          ),
          ctor_id: None,
          super_struct: None,
          is_virtual: false,
        };
        let name = self.reg_base_type(group, reg);
        let item_ = ast::ItemStruct(
          box(GC) struct_def,
          ast::Generics {
            lifetimes: Vec::new(),
            ty_params: owned_slice::OwnedSlice::empty(),
          });
        let attrs = match reg.docstring {
          Some(docstring) => vec!(self.doc_attribute(token::get_ident(docstring.node))),
          None => Vec::new(),
        };
        Some(self.cx.item(reg.name.span, name, attrs, item_))
      },
    }
  }
  
  /// The name of the structure representing a register
  fn reg_base_type(&self, group: P<node::RegGroup>, reg: &node::Reg) -> ast::Ident {
    match reg.ty {
      node::UIntReg(_) => self.cx.ident_of(format!("{}_{}", group.name.node, reg.name.node).as_slice()),
      node::GroupReg(ref g) => {
        if !group.groups.contains_key(g) {
          self.error(reg.name.span, format!("Undefined register group `{}`", g));
          // FIXME: Abort and fix span above
        }
        self.cx.ident_of(g.as_slice())
      }
    }
  }

  /// Returns the type of the field representing the given register in a `RegGroup` struct
  fn reg_struct_type(&self, group: P<node::RegGroup>, reg: &node::Reg) -> P<ast::Ty> {
    let base_ty_path = self.cx.path(DUMMY_SP, vec!(self.reg_base_type(group, reg)));
    let base_ty: P<ast::Ty> = self.cx.ty_path(base_ty_path, None);
    match reg.count.node {
      1 => base_ty,
      n => self.cx.ty(DUMMY_SP, ast::TyFixedLengthVec(base_ty, self.cx.expr_uint(DUMMY_SP, n))),
    }
  }

  /// Produce a field for the given register in a `RegGroup` struct
  fn emit_reg_group_field(&self, group: P<node::RegGroup>, reg: &node::Reg) -> ast::StructField {
    let attrs = match reg.docstring {
      Some(doc) => vec!(),
      None => Vec::new(),
    };
    Spanned {
      span: DUMMY_SP,
      node: ast::StructField_ {
        kind: ast::NamedField(self.cx.ident_of(reg.name.node.as_slice()), ast::Inherited),
        id: ast::DUMMY_NODE_ID,
        ty: self.reg_struct_type(group, reg),
        attrs: attrs,
      }
    }
  }

  fn emit_group(&self, group: P<node::RegGroup>) -> Vec<P<ast::Item>> {
    let reg_structs = group.regs.iter().flat_map(|r| self.emit_reg_struct(group, r).move_iter());
    let struct_def = ast::StructDef {
      fields: FromIterator::from_iter(group.regs.iter().map(|r| self.emit_reg_group_field(group, r))),
      ctor_id: None,
      super_struct: None,
      is_virtual: false,
    };
    let span = DUMMY_SP; // FIXME
    let struct_item = self.cx.item_struct(span, self.cx.ident_of(group.name.node.as_slice()), struct_def);
    let subgroups = group.groups.values().flat_map(|&g| self.emit_group(g).move_iter());
    let hi: Vec<P<ast::Item>> = FromIterator::from_iter(subgroups.chain(reg_structs));
    hi.append_one(struct_item)
  }

  fn error(&self, span: Span, m: String) {
    self.cx.parse_sess().span_diagnostic.span_err(span, m.as_slice());
  }
}

pub fn build_ioregs<'a, 'b>(cx: &'a mut ExtCtxt<'b>, groups: HashMap<String, Gc<node::RegGroup>>) -> Builder<'a, 'b> {
  let builder = Builder::new(cx, groups);
  builder
}
