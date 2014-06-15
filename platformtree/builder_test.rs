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

use syntax;
use syntax::ast;
use syntax::ext::quote::rt::ToSource;
use std::gc::Gc;

use builder::build_platformtree;
use test_helpers::{with_parsed};

#[test]
fn parses_lpc17xx() {
  with_parsed("mcu@lpc17xx;", |cx, failed, pt| {
    let builder = build_platformtree(cx, pt);
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 2);

    assert_equal_source(builder.main_stmts.get(0),
        "{
          use zinc::hal::stack;
          extern \"C\" {
            static _eglobals: u32;
          }
          stack::set_stack_limit((&_eglobals as *u32) as u32);
        }");
    assert_equal_source(builder.main_stmts.get(1),
        "zinc::hal::mem_init::init_data()");
  });
}

// #[test]
// fn parses_lpc17xx_clock() {
//   with_parsed("mcu@lpc17xx {
//       clock {
//         source = \"main-oscillator\";
//         source_frequency = 12_000_000;
//         pll {
//           m = 50;
//           n = 3;
//           divisor = 4;
//         }
//       }
//     }", |cx, failed, pt| {
//     let builder = build_platformtree(cx, pt);
//     assert!(unsafe{*failed} == false);
//     assert!(builder.main_stmts.len() == 2);
//   });
// }

#[test]
fn fails_to_parse_pt_with_anonymous_root_node() {
  with_parsed("node {}", |cx, failed, pt| {
    build_platformtree(cx, pt);
    assert!(unsafe{*failed} == true);
  });
}

#[test]
fn fails_to_parse_pt_with_unknown_root_node() {
  with_parsed("unknown@node {}", |cx, failed, pt| {
    build_platformtree(cx, pt);
    assert!(unsafe{*failed} == true);
  });
}

#[test]
fn fails_to_parse_pt_with_unknown_mcu() {
  with_parsed("mcu@bad {}", |cx, failed, pt| {
    build_platformtree(cx, pt);
    assert!(unsafe{*failed} == true);
  });
}

fn assert_equal_source(stmt: &Gc<syntax::ast::Stmt>, src: &str) {
  let gen_src = match stmt.node {
    ast::StmtExpr(e, _) | ast::StmtSemi(e, _) => e.to_source(),
    _ => fail!(),
  };
  println!("generated: {}", gen_src);
  println!("expected:  {}", src);

  let stripped_gen_src = gen_src.replace(" ", "").replace("\n", "");
  let stripped_src = src.replace(" ", "").replace("\n", "");

  assert!(stripped_gen_src == stripped_src);
}
