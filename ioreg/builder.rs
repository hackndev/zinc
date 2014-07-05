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
use syntax::abi;
use syntax::ast::TokenTree;
use syntax::ast;
use syntax::ast_util::empty_generics;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::parse::token::InternedString;

use node;

pub struct Builder {
  pub type_items: Vec<Gc<ast::Item>>,
  pub ioreg: Gc<node::IoReg>,
}

fn allow_attribute(cx: &mut ExtCtxt, allow: &'static str) -> ast::Attribute {
  let word = cx.meta_word(DUMMY_SP, InternedString::new(allow));
  let allow = cx.meta_list(DUMMY_SP, InternedString::new("allow"), vec!(word));
  cx.attribute(DUMMY_SP, allow)
}

impl Builder {
  pub fn new(ioreg: &Gc<node::IoReg>) -> Builder {
    Builder {
      type_items: Vec::new(),
      ioreg: *ioreg,
    }
  }

  pub fn add_type_item(&mut self, item: Gc<ast::Item>) {
    self.type_items.push(item);
  }

  pub fn emit_items(&self, cx: &mut ExtCtxt) -> Vec<Gc<ast::Item>> {
    //vec!(box(GC) self.emit_type(cx))
    Vec::new()
  }

  //fn emit_type(&self, cx: &mut ExtCtxt) -> ast::Item {}
}

pub fn build_ioreg(cx: &mut ExtCtxt, ioreg: &Gc<node::IoReg>) -> Builder {
  let builder = Builder::new(ioreg);
  builder
}
