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

use std::gc::Gc;
use syntax::ast;
use syntax::codemap::{ExpnInfo, NameAndSpan, MacroBang};
use syntax::ext::base::ExtCtxt;

use node;

pub struct Builder {
  pub main_stmts: Vec<Gc<ast::Stmt>>,
}

impl Builder {
  pub fn new() -> Builder {
    Builder {
      main_stmts: Vec::new(),
    }
  }

  pub fn add_main_statement(&mut self, stmt: Gc<ast::Stmt>) {
    self.main_stmts.push(stmt);
  }
}

pub fn build_platformtree(cx: &mut ExtCtxt, pt: &node::PlatformTree) -> Builder {
  let mut builder = Builder::new();

  for n in pt.iter() {
    match n.name {
      None => {
        cx.parse_sess().span_diagnostic.span_err(n.name_span,
            "root node cannot be anonymous");
        continue;
      },
      Some(ref name) => {
        match name.as_slice() {
          "mcu" => {
            build_mcu(&mut builder, cx, n);
          },
          other => {
            cx.parse_sess().span_diagnostic.span_err(n.name_span,
                format!("unknown root node `{}`", other).as_slice());
            continue;
          }
        }
      },
    }
  }

  builder
}

fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt, node: &Gc<node::Node>) {
  cx.bt_push(ExpnInfo {
    call_site: node.name_span,
    callee: NameAndSpan {
      name: "platformtree".to_str(),
      format: MacroBang,
      span: None,
    },
  });

  match node.path.as_slice() {
    "lpc17xx" => lpc17xx_pt::build_mcu(builder, cx, node),
    other => {
      cx.parse_sess().span_diagnostic.span_err(node.name_span,
          format!("unknown mcu `{}`", other).as_slice());
    },
  }

  cx.bt_pop();
}

mod lpc17xx_pt {
  use std::gc::Gc;
  use syntax::ext::base::ExtCtxt;
  use syntax::ext::build::AstBuilder;
  use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
  use syntax::ast::TokenTree;

  use super::Builder;
  use node;


  pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
      node: &Gc<node::Node>) {
    if !node.expect_no_attributes(cx) {
      return;
    }
    // init stack
    builder.add_main_statement(cx.stmt_expr(quote_expr!(&*cx,
        {
          use zinc::hal::stack;
          extern { static _eglobals: u32; }
          stack::set_stack_limit((&_eglobals as *u32) as u32);
        }
    )));

    // init data
    builder.add_main_statement(cx.stmt_expr(quote_expr!(&*cx,
        zinc::hal::mem_init::init_data();
    )));

    node.get_by_path("clock").and_then(|sub| -> Option<bool> {
      build_clock(builder, cx, sub);
      None
    });
  }

  pub fn build_clock(builder: &mut Builder, cx: &mut ExtCtxt,
      node: &Gc<node::Node>) {
    if !node.expect_attributes(cx, vec!(("source", node::StringAttribute))) {
      return;
    }

    let source = node.get_string_attr("source").unwrap();
    let clock_source = ClockSource::new(match source.as_slice() {
      "internal-oscillator" => "init::Internal".to_str(),
      "rtc-oscillator"      => "init::RTC".to_str(),
      "main-oscillator"     => {
        let some_source_frequency =
            node.get_required_int_attr(cx, "source_frequency");
        if some_source_frequency == None {
          "BAD".to_str()
        } else {
          format!("init::Main({})", some_source_frequency.unwrap())
        }
      },
      other => {
        cx.span_err(
            node.get_attr("source").value_span,
            format!("unknown oscillator value `{}`", other).as_slice());
        "BAD".to_str()
      },
    });

    let some_pll_conf = node.get_by_path("pll").and_then(|sub|
        -> Option<(uint, uint, uint)> {
      if !sub.expect_no_subnodes(cx) || !sub.expect_attributes(cx, vec!(
          ("m", node::IntAttribute),
          ("n", node::IntAttribute),
          ("divisor", node::IntAttribute))) {
        None
      } else {
        let m = sub.get_int_attr("m").unwrap();
        let n = sub.get_int_attr("n").unwrap();
        let divisor = sub.get_int_attr("divisor").unwrap();
        Some((m, n, divisor))
      }
    });
    if some_pll_conf.is_none() {
      cx.parse_sess().span_diagnostic.span_err(node.name_span,
          "required subnode `pll` is missing");
      return;
    }

    let (m, n, divisor) = some_pll_conf.unwrap();

    let ex = quote_expr!(&*cx,
        {
          use zinc::hal::lpc17xx::init;
          init::init_clock(
              init::Clock {
                source: $clock_source,
                pll: init::PLL0 {
                  enabled: true,
                  m: $m,
                  n: $n,
                  divisor: $divisor,
                }
              }
          );
        }
    );
    builder.add_main_statement(cx.stmt_expr(ex));
  }

  struct ClockSource {
    pub s: String,
  }

  impl ClockSource {
    pub fn new(s: String) -> ClockSource {
      ClockSource {
        s: s,
      }
    }
  }

  impl ToTokens for ClockSource {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
      (cx as &ExtParseUtils).parse_tts(self.s.clone())
    }
  }
}
