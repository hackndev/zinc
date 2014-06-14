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
use syntax::ast::TokenTree;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};

use node;
use super::PlatformContext;

/// Entry point for parsing an mcu node.
///
/// Arguments:
///   pcx:   PlatformContext, used to generate any static items in
///          mod platformtree
///   ecx:   ExtCtxt for building ASTs
///   nodes: a vector of nodes inside of the mcu node.
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

/// A simple wrapper to allow custom tokenization of ClockSource
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

/// Parses @clock node into pll init code.
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

  let clock_source = ClockSource::new(match source.as_slice() {
    "internal-oscillator" => "init::Internal".to_str(),
    "main-oscillator"     => {
      let source_frequency = node.unwrap_int(ecx, "source_frequency");
      if source_frequency == None {
        return
      }
      format!("init::Main({})", source_frequency)
    }
    "rtc-oscillator"      => "init::RTC".to_str(),
    other => {
      ecx.span_err(
          node.path.span.unwrap(), // TODO: span
          format!("unknown oscillator value `{}`", other).as_slice());
      return
    },
  });

  let ex = quote_expr!(&*ecx,
      {
        use zinc::hal::lpc17xx::init;
        init::init_clock(
            init::Clock {
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
  pcx.add_main_statement(ecx.stmt_expr(ex));
}
