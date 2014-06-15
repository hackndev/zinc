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

#![crate_id="macro_platformtree"]
#![crate_type="dylib"]

#![feature(plugin_registrar, quote, managed_boxes)]

extern crate rustc;
extern crate syntax;
extern crate platformtree;

use rustc::plugin::Registry;
use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::ext::base;
use syntax::ext::quote::rt::{ToTokens, ToSource, ExtParseUtils};

use platformtree::parser::Parser;
use platformtree::context;

/// Register available macros.
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("platformtree_parse", platformtree_parse);
}

/// platformtree_parse parses a platfrom tree into node::Node struct.
pub fn platformtree_parse(cx: &mut ExtCtxt, _: Span, tts: &[TokenTree])
    -> Box<MacResult> {
  base::MacExpr::new(quote_expr!(&*cx, true))
}
