// Zinc, the bare metal stack for rust.
// Copyright 2014 Matt "mcoffin" Coffin <mcoffin13@gmail.com>
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

#![feature(rustc_private, plugin_registrar, quote)]

extern crate rustc;
extern crate syntax;
extern crate rustc_plugin;

use rustc_plugin::Registry;
use syntax::ast::MetaItem;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{Annotatable, ExtCtxt, MultiDecorator};
use syntax::ext::build::AstBuilder;

macro_rules! and_return {
  ($a:stmt) => (
    {
      $a;
      return;
    }
    )
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_syntax_extension(syntax::parse::token::intern("zinc_main"),
  MultiDecorator(Box::new(decorator_zinc_main)));
}

pub fn decorator_zinc_main(cx: &mut ExtCtxt,
                           sp: Span,
                           _: &MetaItem,
                           item: &Annotatable,
                           push: &mut FnMut(Annotatable)) {
  let main = match item {
    &Annotatable::Item(ref main_item) => (*main_item).ident,
    _ => and_return!(cx.span_err(sp, "zinc_main must be an item")),
  };

  let call_main = cx.expr_call_ident(DUMMY_SP, main, Vec::new());
  let start = quote_item!(cx,
    #[start]
    fn start(_: isize, _: *const *const u8) -> isize {
      $call_main;
      0
    }
  ).unwrap();
  push(Annotatable::Item(start));
}
