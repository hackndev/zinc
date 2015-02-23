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

use builder::{Builder, TokenString, add_node_dependency};
use node;

pub fn attach(builder: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  for timer_node in node.subnodes().iter() {
    timer_node.materializer.set(Some(build_timer as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
    add_node_dependency(&node, timer_node);
    super::add_node_dependency_on_clock(builder, timer_node);
  }
}

pub fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
}

fn build_timer(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  if !node.expect_attributes(cx, &[
      ("counter", node::IntAttribute),
      ("divisor", node::IntAttribute)]) {
    return
  }

  if node.name.is_none() {
    cx.parse_sess().span_diagnostic.span_err(node.name_span,
        "timer node must have a name");
    return
  }

  let name = TokenString(node.name.clone().unwrap());
  let timer_index: usize = node.path.as_slice().parse().unwrap();
  let counter: u32 = node.get_int_attr("counter").unwrap() as u32;
  let divisor: u8 = node.get_int_attr("divisor").unwrap() as u8;

  let timer_name = match timer_index {
    0...3 => TokenString(format!(
        "zinc::hal::lpc17xx::timer::TimerPeripheral::Timer{}", timer_index)),
    other => {
      cx.parse_sess().span_diagnostic.span_err(node.path_span,
          format!("unknown timer index `{}`, allowed indexes: 0, 1, 2, 3",
              other).as_slice());
      return
    }
  };

  node.set_type_name("zinc::hal::lpc17xx::timer::Timer".to_string());

  let st = quote_stmt!(&*cx,
      let $name = zinc::hal::lpc17xx::timer::Timer::new(
          $timer_name, $counter, $divisor);
  );
  builder.add_main_statement(st);
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
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
      let mut builder = Builder::new(pt.clone(), cx);
      super::build_timer(&mut builder, cx, pt.get_by_name("tim").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts().len() == 1);

      assert_equal_source(builder.main_stmts()[0].deref(),
          "let tim = zinc::hal::lpc17xx::timer::Timer::new(
              zinc::hal::lpc17xx::timer::TimerPeripheral::Timer1, 25u32, 4u8);");
    });
  }
}
