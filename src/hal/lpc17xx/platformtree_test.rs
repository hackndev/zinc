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

use builder::build_platformtree;
use test_helpers::{assert_equal_source, with_parsed, fails_to_build};

#[test]
fn builds_stack_data_init() {
  with_parsed("lpc17xx@mcu;", |cx, failed, pt| {
    let builder = build_platformtree(cx, pt);
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 0);
  });
}

#[test]
fn fails_to_parse_garbage_attrs() {
  fails_to_build("lpc17xx@mcu { key = 1; }");
}

#[test]
fn builds_clock_init() {
  with_parsed("lpc17xx@mcu {
      clock {
        source = \"main-oscillator\";
        source_frequency = 12_000_000;
        pll {
          m = 50;
          n = 3;
          divisor = 4;
        }
      }
    }", |cx, failed, pt| {
    let builder = build_platformtree(cx, pt);
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 1);

    assert_equal_source(builder.main_stmts.get(0),
        "{
          use zinc::hal::lpc17xx::init;
          init::init_clock(
              init::Clock {
                source: init::Main(12000000),
                pll: init::PLL0 {
                  enabled: true,
                  m: 50u8,
                  n: 3u8,
                  divisor: 4u8,
                },
              }
          );
        }");
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
