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
use syntax::abi;
use syntax::ast::TokenTree;
use syntax::ast;
use syntax::ast_util::empty_generics;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::owned_slice;
use syntax::parse::token::InternedString;

use node;

pub struct Builder {
  pub type_items: Vec<Gc<ast::Item>>,
  pub groups: HashMap<String, Gc<node::RegGroup>>,
}

/// Generate an `#[allow(...)]` attribute of the given type
fn allow_attribute(cx: &mut ExtCtxt, allow: &'static str) -> ast::Attribute {
  let word = cx.meta_word(DUMMY_SP, InternedString::new(allow));
  let allow = cx.meta_list(DUMMY_SP, InternedString::new("allow"), vec!(word));
  cx.attribute(DUMMY_SP, allow)
}

impl Builder {
  pub fn new(groups: HashMap<String, Gc<node::RegGroup>>) -> Builder {
    Builder {
      type_items: Vec::new(),
      groups: groups,
    }
  }

  pub fn add_type_item(&mut self, item: Gc<ast::Item>) {
    self.type_items.push(item);
  }

  pub fn emit_items(&self, cx: &mut ExtCtxt) -> Vec<Gc<ast::Item>> {
    //vec!(box(GC) self.emit_type(cx))
    Vec::new()
  }

  /*
  fn emit_type(&self, cx: &mut ExtCtxt) -> ast::Item {
    let span: Span = undefined;
    let width: uint = self.ioreg.width();
    if width % 8 != 0 {
      println!("Not multiple of byte size\n");
    }
    let prim_ty: ast::Path = undefined;
    let ty: Gc<ast::Ty> = box(GC) ast::Ty {
      id: ast::DUMMY_NODE_ID,
      node: ast::TyPath(prim_ty, None, ast::DUMMY_NODE_ID),
      span: span
    };
    ast::Item {
      ident: cx.ident_of(self.ioreg.name),
      attrs: vec!(allow_attribute(cx, "uppercase_variables")),
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemTy(ty, ast::Generics { lifetimes: Vec::new(),
                                            ty_params: owned_slice::OwnedSlice::empty() }),
      vis: ast::Public,
      span: span,
    }
  }
  */
}

pub fn build_ioregs(cx: &mut ExtCtxt, groups: HashMap<String, Gc<node::RegGroup>>) -> Builder {
  let builder = Builder::new(groups);
  builder
}
