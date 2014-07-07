// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
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

#![crate_name="macro_platformtree"]
#![crate_type="dylib"]

#![feature(plugin_registrar, quote, managed_boxes)]

extern crate rustc;
extern crate syntax;
extern crate platformtree;

use rustc::plugin::Registry;
use std::gc::Gc;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::print::pprust;
use syntax::util::small_vector::SmallVector;

use platformtree::parser::Parser;
use platformtree::builder::build_platformtree;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("platformtree", macro_platformtree);
  reg.register_macro("platformtree_verbose", macro_platformtree_verbose);
}

pub fn macro_platformtree(cx: &mut ExtCtxt, _: Span, tts: &[ast::TokenTree])
    -> Box<MacResult> {
  let pt = Parser::new(cx, tts).parse_platformtree();
  let builder = build_platformtree(cx, &pt.unwrap());

  let items = builder.emit_items(cx);
  MacItems::new(items)
}


pub fn macro_platformtree_verbose(cx: &mut ExtCtxt, sp: Span,
    tts: &[ast::TokenTree]) -> Box<MacResult> {
  let result = macro_platformtree(cx, sp, tts);

  println!("Platform Tree dump:")
  for i in result.make_items().unwrap().as_slice().iter() {
    println!("{}", pprust::item_to_str(i.deref()));
  }

  result
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
