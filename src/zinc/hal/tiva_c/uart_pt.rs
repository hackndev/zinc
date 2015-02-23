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

  for sub in node.subnodes().iter() {
    add_node_dependency(&node, sub);
    super::add_node_dependency_on_clock(builder, sub);

    sub.materializer.set(Some(build_uart as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  }
}

pub fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
}

pub fn build_uart(builder: &mut Builder,
                  cx: &mut ExtCtxt,
                  sub: Rc<node::Node>) {
  let error = |&: err: &str | {
    cx.parse_sess().span_diagnostic.span_err(sub.path_span, err);
  };

  let uart_peripheral_str = format!("Uart{}",
      match sub.path.as_slice().parse::<usize>().unwrap() {
        0 ... 7 => sub.path.clone(),
        p       => {
          error(format!("unknown UART `{}`, allowed values: 0, 2, 3",
                        p).as_slice());
          return;
        }
      });
  let uart_peripheral = TokenString(uart_peripheral_str);

  if sub.name.is_none() {
    cx.parse_sess().span_diagnostic.span_err(sub.name_span,
        "UART node must have a name");
    return
  }

  if !sub.expect_attributes(cx, &[("mode", node::StrAttribute)]) {
    return
  }

  let mode = sub.get_string_attr("mode").unwrap();

  let mode_re =
    Regex::new(r"([[:digit:]]+),?([[:digit:]]*)([nNoOEe]?)([[:digit:]])?").unwrap();

  let mode_captures = match mode_re.captures(mode.as_slice()) {
    Some(c) => c,
    None    => {
      error(format!("invalid format {}", mode).as_slice());
      return;
    }
  };

  let baud_rate = mode_captures.at(1).unwrap().parse::<usize>();

  let word_len = match mode_captures.at(2).unwrap() {
    "" => 8,
    l  => l.parse::<u8>().unwrap(),
  };

  let parity = TokenString(match mode_captures.at(3).unwrap() {
    ""|"N"|"n" => "Disabled",
    "O"|"o"    => "Odd",
    "E"|"e"    => "Even",
    "1"        => "Forced1",
    "0"        => "Forced0",
    p          => {
      error(format!("invalid parity setting {}", p).as_slice());
      return;
    }
  }.to_string());

  let stop_bits = match mode_captures.at(4).unwrap() {
    "" => 1,
    s  => s.parse::<u8>().unwrap(),
  };

  sub.set_type_name("zinc::hal::tiva_c::uart::Uart".to_string());
  let uart_name = TokenString(sub.name.clone().unwrap());

  let st = quote_stmt!(&*cx,
      let $uart_name = zinc::hal::tiva_c::uart::Uart::new(
          zinc::hal::tiva_c::uart::UartId::$uart_peripheral,
          $baud_rate,
          $word_len,
          zinc::hal::uart::Parity::$parity,
          $stop_bits)
  );

  builder.add_main_statement(st);
}
