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
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::ast::TokenTree;

use builder::Builder;
use node;

pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

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
  let pll_m: u8 = m as u8;
  let pll_n: u8 = n as u8;
  let pll_divisor: u8 = divisor as u8;

  let ex = quote_expr!(&*cx,
      {
        use zinc::hal::lpc17xx::init;
        init::init_clock(
            &init::Clock {
              source: $clock_source,
              pll: init::PLL0 {
                enabled: true,
                m: $pll_m,
                n: $pll_n,
                divisor: $pll_divisor,
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
