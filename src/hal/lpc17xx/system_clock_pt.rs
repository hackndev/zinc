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

use std::rc::Rc;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;

use builder::{Builder, TokenString};
use node;

pub fn attach(_: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(build_clock as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
}

fn build_clock(builder: &mut Builder, cx: &mut ExtCtxt,
    node: Rc<node::Node>) {
  if !node.expect_attributes(cx, &[("source", node::StrAttribute)]) {
    return;
  }

  let source = node.get_string_attr("source").unwrap();
  let source_freq: usize;
  let clock_source = TokenString(match source.as_str() {
    "internal-oscillator" => {
      source_freq = 4_000_000;
      "system_clock::Internal".to_string()
    },
    "rtc-oscillator"      => {
      source_freq = 32_000;
      "system_clock::RTC".to_string()
    },
    "main-oscillator"     => {
      let some_source_frequency =
          node.get_required_int_attr(cx, "source_frequency");
      if some_source_frequency == None {
        source_freq = 0;
        "BAD".to_string()
      } else {
        source_freq = some_source_frequency.unwrap();
        format!("system_clock::ClockSource::Main({})", source_freq)
      }
    },
    other => {
      source_freq = 0;
      cx.span_err(
          node.get_attr("source").value_span,
          format!("unknown oscillator value `{}`", other).as_str());
      "BAD".to_string()
    },
  });

  let some_pll_conf = node.get_by_path("pll").and_then(|sub|
      -> Option<(usize, usize, usize)> {
    if !sub.expect_no_subnodes(cx) || !sub.expect_attributes(cx, &[
        ("m", node::IntAttribute),
        ("n", node::IntAttribute),
        ("divisor", node::IntAttribute)]) {
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

  let sysfreq = source_freq * 2 * pll_m as usize / pll_n as usize
      / pll_divisor as usize;
  node.attributes.borrow_mut().insert("system_frequency".to_string(),
      Rc::new(node::Attribute::new_nosp(node::IntValue(sysfreq))));

  let ex = quote_expr!(&*cx,
      {
        use zinc::hal::lpc17xx::system_clock;
        system_clock::init_clock(
            &system_clock::Clock {
              source: $clock_source,
              pll: core::option::Option::Some(system_clock::PLL0 {
                m: $pll_m,
                n: $pll_n,
                divisor: $pll_divisor,
              })
            }
        );
      }
  );
  builder.add_main_statement(cx.stmt_expr(ex));
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
  use builder::Builder;
  use test_helpers::{assert_equal_source, with_parsed, fails_to_build};

  #[test]
  fn builds_clock_init() {
    with_parsed("
      clock {
        source = \"main-oscillator\";
        source_frequency = 12_000_000;
        pll {
          m = 50;
          n = 3;
          divisor = 4;
        }
      }", |cx, failed, pt| {
      let mut builder = Builder::new(pt.clone(), cx);
      super::build_clock(&mut builder, cx, pt.get_by_path("clock").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts().len() == 1);

      assert_equal_source(builder.main_stmts()[0].deref(),
          "{
            use zinc::hal::lpc17xx::system_clock;
            system_clock::init_clock(
                &system_clock::Clock {
                  source: system_clock::ClockSource::Main(12000000),
                  pll: core::option::Option::Some(system_clock::PLL0 {
                    m: 50u8,
                    n: 3u8,
                    divisor: 4u8,
                  }),
                }
            );
          };");
    });
  }

  #[test]
  fn clock_provides_out_frequency() {
    with_parsed("
      clock {
        source = \"main-oscillator\";
        source_frequency = 12_000_000;
        pll {
          m = 50;
          n = 3;
          divisor = 4;
        }
      }", |cx, _, pt| {
      let mut builder = Builder::new(pt.clone(), cx);
      let node = pt.get_by_path("clock").unwrap();
      super::build_clock(&mut builder, cx, node.clone());

      let out_freq = node.get_int_attr("system_frequency");
      assert!(out_freq.is_some());
      assert!(out_freq.unwrap() == 100_000_000);
    });
  }

  #[test]
  fn fails_to_parse_bad_clock_conf() {
    fails_to_build("lpc17xx@mcu { clock {
      no_source = 1;
      source_frequency = 12_000_000;
    }}");
    fails_to_build("lpc17xx@mcu { clock {
      source = \"missing\";
      source_frequency = 12_000_000;
    }}");
  }

  #[test]
  fn fails_to_parse_no_pll_clock() {
    fails_to_build("lpc17xx@mcu { clock {
      source = \"main-oscillator\";
      source_frequency = 12_000_000;
    }}");
  }
}
