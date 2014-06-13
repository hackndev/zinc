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

use syntax::ast;
use syntax::codemap::Span;
use std::gc::Gc;
use syntax::ext::base::ExtCtxt;
use syntax::codemap::{mk_sp, BytePos};
use syntax::ext::build::AstBuilder;

use node;

pub struct PlatformContext {
  mod_items: Vec<Gc<ast::Item>>,
  main_stmts: Vec<Gc<ast::Stmt>>,
}

impl PlatformContext {
  pub fn new<'a>() -> PlatformContext {
    PlatformContext {
      mod_items: Vec::new(),
      main_stmts: Vec::new(),
    }
  }

  pub fn add_item(&mut self, item: Gc<ast::Item>) {
    self.mod_items.push(item);
  }

  pub fn add_main_statement(&mut self, stmt: Gc<ast::Stmt>) {
    self.main_stmts.push(stmt);
  }

  pub fn get_main_block(&self, ecx: &ExtCtxt) -> ast::P<ast::Block> {
    let sp = mk_sp(BytePos(0), BytePos(0));
    ecx.block(sp, self.main_stmts.clone(), None)
  }
}

pub fn process_node(pcx: &mut PlatformContext, ecx: &ExtCtxt, node: Gc<node::Node>) {
  match node.path.path.get(0).as_slice() {
    "mcu" => process_mcu(pcx, ecx, node.path.path.get(1), node),
    other => ecx.span_err(
        node.path.span.unwrap(),
        format!("unknown root path `{}`", other).as_slice()),
  }
}

fn process_mcu(pcx: &mut PlatformContext, ecx: &ExtCtxt, mcu: &String, node: Gc<node::Node>) {
  match mcu.as_slice() {
    "lpc17xx" => lpc17xx_pt::process_nodes(pcx, ecx, &node.subnodes),
    other => ecx.span_err(
        node.path.span.unwrap(),
        format!("unknown mcu `{}`", other).as_slice()),
  }
}

impl node::Node {
  pub fn unwrap_string(&self, ecx: &ExtCtxt, attr: &str) -> Option<String> {
    match self.unwrap_attribute(ecx, attr) {
      Some(a) => match a {
        node::StrValue(v) => Some(v),
        _ => {
          ecx.span_err(
              self.path.span.unwrap(),  // TODO: wrong span
              format!("required string key `{}` is not of a string type", attr).as_slice());
          None
        }
      },
      None => None,
    }
  }

  pub fn unwrap_int(&self, ecx: &ExtCtxt, attr: &str) -> Option<uint> {
    match self.unwrap_attribute(ecx, attr) {
      Some(a) => match a {
        node::UIntValue(v) => Some(v),
        _ => {
          ecx.span_err(
              self.path.span.unwrap(),  // TODO: wrong span
              format!("required string key `{}` is not of a string type", attr).as_slice());
          None
        }
      },
      None => None,
    }
  }

  fn unwrap_attribute(&self, ecx: &ExtCtxt, attr: &str) -> Option<node::AttributeValue> {
    match self.attributes.find_equiv(&attr.to_str()) {
      Some(a) => Some(a.clone()),
      None => {
        ecx.span_err(
            self.path.span.unwrap(),  // TODO: wrong span
            format!("required attribute `{}` is missing", attr).as_slice());
        None
      }
    }
  }
}

mod lpc17xx_pt {
  use syntax::codemap::Span;
  use std::gc::Gc;
  use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
  use syntax::ext::base::ExtCtxt;
  use syntax::ast::TokenTree;
  use syntax::ext::build::AstBuilder;

  use super::PlatformContext;
  use node;

  pub fn process_nodes(pcx: &mut PlatformContext, ecx: &ExtCtxt, nodes: &Vec<Gc<node::Node>>) {
    for n in nodes.iter() {
      let path = n.path.path.get(0).as_slice();
      match path {
        "clock" => process_clock(pcx, ecx, n),
        other => ecx.span_err(
            n.path.span.unwrap(),
            format!("unknown subnode `{}` in lpc17xx mcu", other).as_slice()),
      }
    }
  }

  struct TokenSource {
    pub s: String,
  }

  impl ToTokens for TokenSource {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
      (cx as &ExtParseUtils).parse_tts(self.s.clone())
    }
  }

  fn process_clock(pcx: &mut PlatformContext, ecx: &ExtCtxt, node: &Gc<node::Node>) {
    if node.path.path.len() != 1 {
      ecx.span_err(node.path.span.unwrap(), "node lpc17xx::clock is final");
      return
    }

    let some_source = node.unwrap_string(ecx, "source");
    let some_pll_m = node.unwrap_int(ecx, "pll_m");
    let some_pll_n = node.unwrap_int(ecx, "pll_n");
    let some_pll_divisor = node.unwrap_int(ecx, "pll_divisor");

    if some_source == None || some_pll_m == None || some_pll_m == None ||
        some_pll_divisor == None {
      return
    }

    let source = some_source.unwrap();
    let pll_m = some_pll_m.unwrap();
    let pll_n = some_pll_n.unwrap();
    let pll_divisor = some_pll_divisor.unwrap();

    let token_source = TokenSource { s: match source.as_slice() {
      "internal-oscillator" => "zinc::hal::lpc17xx::init::Internal".to_str(),
      "main-oscillator"     => {
        let source_frequency = node.unwrap_int(ecx, "source_frequency");
        if source_frequency == None {
          return
        }
        format!("zinc::hal::lpc17xx::init::Main({})", source_frequency)
      }
      "rtc-oscillator"      => "zinc::hal::lpc17xx::init::RTC".to_str(),
      other => {
        ecx.span_err(
            node.path.span.unwrap(), // TODO: span
            format!("unknown oscillator value `{}`", other).as_slice());
        return
      },
    }};

    let ex = quote_expr!(&*ecx,
        zinc::hal::lpc17xx::init::init_clock(
            zinc::hal::lpc17xx::init::Clock {
              source: $token_source,
              pll: zinc::hal::lpc17xx::init::PLL0 {
                enabled: true,
                m: $pll_m,
                n: $pll_n,
                divisor: $pll_divisor,
              }
            }
        );
    );
    pcx.add_main_statement(ecx.stmt_expr(ex));
  }
}
