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

#![crate_name="macro_ioreg"]
#![crate_type="dylib"]

#![feature(plugin_registrar, quote, managed_boxes)]

extern crate rustc;
extern crate syntax;
extern crate ioreg;

use rustc::plugin::Registry;
use std::gc::Gc;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::util::small_vector::SmallVector;

use ioreg::parser::Parser;
use ioreg::builder::Builder;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("ioregs", macro_ioregs);
}

pub fn macro_ioregs(cx: &mut ExtCtxt, _: Span, tts: &[ast::TokenTree])
                    -> Box<MacResult> {
  match Parser::new(cx, tts).parse_ioregs() {
    Some(group) => {
      let mut builder = Builder::new();
      let items = builder.emit_items(cx, group);
      MacItems::new(items)
    },
    None => {
      fail!();
    }
  }
}

pub struct MacItems {
  items: Vec<Gc<ast::Item>>
}

impl MacItems {
  pub fn new(items: Vec<Gc<ast::Item>>) -> Box<MacResult> {
    box MacItems { items: items } as Box<MacResult>
  }
}

impl MacResult for MacItems {
  fn make_items(&self) -> Option<SmallVector<Gc<ast::Item>>> {
    Some(SmallVector::many(self.items.clone()))
  }
}
