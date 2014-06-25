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

use syntax::codemap::DUMMY_SP;
use syntax::ext::build::AstBuilder;

use builder::{Builder, build_os};
use test_helpers::{assert_equal_source, with_parsed, fails_to_build};

#[test]
fn fails_to_parse_pt_with_unknown_root_node() {
  fails_to_build("unknown@node {}");
}

#[test]
fn fails_to_parse_pt_with_unknown_mcu() {
  fails_to_build("mcu@bad {}");
}

#[test]
fn builds_single_task_os_loop() {
  with_parsed("os {
      single_task {
        loop = \"run\";
      }
    }", |cx, failed, pt| {
    let mut builder = Builder::new(pt);
    build_os(&mut builder, cx, pt.get_by_path("os").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 1);

    assert_equal_source(builder.main_stmts.get(0),
        "loop {
          run();
        }");
  });
}

#[test]
fn builds_single_task_with_args() {
  with_parsed("os {
      single_task {
        loop = \"run\";
        args {
          a = 1;
          b = \"a\";
          c = &named;
        }
      }
    }

    named@ref;
    ", |cx, failed, pt| {
    let mut builder = Builder::new(pt);
    pt.get_by_path("ref").unwrap().type_name.set(Some("hello::world::Struct"));

    build_os(&mut builder, cx, pt.get_by_path("os").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 1);
    assert!(builder.type_items.len() == 1);

    assert_equal_source(&cx.stmt_item(DUMMY_SP, *builder.type_items.get(0)),
        "pub struct run_args<'a> {
          pub a: u32,
          pub b: &'static str,
          pub c: &'a hello::world::Struct,
        }");

    assert_equal_source(builder.main_stmts.get(0),
        "loop {
          run(&pt::run_args {
            a: 1u,
            b: \"a\",
            c: &named,
          });
        }");
  });
}
