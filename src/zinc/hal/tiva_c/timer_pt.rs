// Zinc, the bare metal stack for rust.
// Copyright 2014 Lionel Flandrin <lionel@svkt.org>
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
use regex::Regex;

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

  let error = |&: err: &str | {
    cx.parse_sess().span_diagnostic.span_err(node.path_span, err);
  };

  if !node.expect_attributes(cx, &[
      ("prescale", node::IntAttribute),
      ("mode", node::StrAttribute)]) {
    return
  }

  if node.name.is_none() {
    cx.parse_sess().span_diagnostic.span_err(node.name_span,
        "timer node must have a name");
    return
  }

  let name = TokenString(node.name.clone().unwrap());
  let prescale = node.get_int_attr("prescale").unwrap() as u32;
  let mode = node.get_string_attr("mode").unwrap();

  // Timer path is in the form "w?[0-5][A-B]":
  // - 'w' denotes a wide (32/64bit) counter.
  // - The number is the timer ID.
  // - The letter says which counter to use within that timer (each timer has
  //   two counters, A and B which can be configured independantly.

  let (wide_timer, id) =
    match Regex::new(r"(w?)([0-5])").unwrap().captures(node.path.as_slice()) {
      Some(c) => {
        (c.at(1) != Some(""), c.at(2))
      }
      None => {
        error(
          format!("invalid timer index `{}`, it should match `w?[0-5]`",
                  node.path).as_slice());
        return;
      }
  };

  let mode = TokenString(
    format!("zinc::hal::tiva_c::timer::Mode::{}",
            match mode.as_slice() {
              "periodic"   => "Periodic",
              "one-shot"   => "OneShot",
              "RTC"        => "RTC",
              "edge-count" => "EdgeCount",
              "edge-time"  => "EdgeTime",
              "PWM"        => "PWM",
              _            => {
                error(format!("unknown mode {}, expected one of \
                               periodic, one-shot, RTC, edge-count, edge-time \
                               or PWM", mode).as_slice());
                return;
              }}));

  let timer_name = TokenString(
    format!("zinc::hal::tiva_c::timer::TimerId::{}{}",
            if wide_timer {
              "TimerW"
            } else {
              "Timer"
            }, id.unwrap()));

  node.set_type_name("zinc::hal::tiva_c::timer::Timer".to_string());

  let st = quote_stmt!(&*cx,
      let $name = zinc::hal::tiva_c::timer::Timer::new(
          $timer_name, $mode, $prescale);
  );
  builder.add_main_statement(st);
}
