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

use builder::{Builder, TokenString};
use node;

pub fn build_timer(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  for (path, sub) in node.subnodes.iter() {
    if !sub.expect_attributes(cx, [
        ("counter", node::IntAttribute),
        ("divisor", node::IntAttribute)]) {
      continue;
    }

    if sub.name.is_none() {
      cx.parse_sess().span_diagnostic.span_err(sub.name_span,
          "timer node must have a name");
      continue;
    }

    let name = TokenString(sub.name.clone().unwrap());
    let timer_index: uint = from_str(path.as_slice()).unwrap();
    let counter: u32 = sub.get_int_attr("counter").unwrap() as u32;
    let divisor: u8 = sub.get_int_attr("divisor").unwrap() as u8;

    let timer_name = match timer_index {
      0..3 => TokenString(format!(
          "zinc::hal::lpc17xx::timer::Timer{}", timer_index)),
      other => {
        cx.parse_sess().span_diagnostic.span_err(sub.path_span,
            format!("unknown timer index `{}`, allowed indexes: 0, 1, 2, 3",
                other).as_slice());
        continue;
      }
    };

    sub.type_name.set(Some("zinc::hal::lpc17xx::timer::Timer"));

    let st = quote_stmt!(&*cx,
        let $name = zinc::hal::lpc17xx::timer::Timer::new(
            $timer_name, $counter, $divisor);
    );
    builder.add_main_statement(st);
  }
}

#[cfg(test)]
mod test {
  use builder::Builder;
  use test_helpers::{assert_equal_source, with_parsed};

  #[test]
  fn builds_timer() {
    with_parsed("
      timer {
        tim@1 {
          counter = 25;
          divisor = 4;
        }
      }", |cx, failed, pt| {
      let mut builder = Builder::new(pt);
      super::build_timer(&mut builder, cx, pt.get_by_path("timer").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);

      assert_equal_source(builder.main_stmts.get(0),
          "let tim = zinc::hal::lpc17xx::timer::Timer::new(
              zinc::hal::lpc17xx::timer::Timer1, 25u32, 4u8);");
    });
  }
}
