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

  for sub in node.subnodes().iter() {
    add_node_dependency(&node, sub);
    let tx_node_name = sub.get_ref_attr("tx").unwrap();
    let rx_node_name = sub.get_ref_attr("rx").unwrap();
    let tx_node = builder.pt().get_by_name(tx_node_name.as_str()).unwrap();
    let rx_node = builder.pt().get_by_name(rx_node_name.as_str()).unwrap();
    add_node_dependency(sub, &tx_node);
    add_node_dependency(sub, &rx_node);
    super::add_node_dependency_on_clock(builder, sub);

    sub.materializer.set(Some(build_uart as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
    sub.mutator.set(Some(mutate_pins as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  }
}

pub fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
}

pub fn mutate_pins(builder: &mut Builder, _: &mut ExtCtxt, sub: Rc<node::Node>) {
  let tx_node_name = sub.get_ref_attr("tx").unwrap();
  let rx_node_name = sub.get_ref_attr("rx").unwrap();

  build_uart_gpio(builder, sub.path.as_str().parse().unwrap(),
      tx_node_name.as_str(), true);
  build_uart_gpio(builder, sub.path.as_str().parse().unwrap(),
      rx_node_name.as_str(), false);
}

pub fn build_uart(builder: &mut Builder, cx: &mut ExtCtxt,
    sub: Rc<node::Node>) {
  let uart_peripheral_str = format!("UARTPeripheral::UART{}",
      match sub.path.as_str().parse::<usize>().unwrap() {
        0|2|3 => sub.path.clone(),
        other => {
          cx.parse_sess().span_diagnostic.span_err(sub.path_span,
              format!("unknown UART `{}`, allowed values: 0, 2, 3",
                  other).as_str());
          return
        }
      });
  let uart_peripheral = TokenString(uart_peripheral_str);

  if sub.name.is_none() {
    cx.parse_sess().span_diagnostic.span_err(sub.name_span,
        "UART node must have a name");
    return
  }

  if !sub.expect_attributes(cx, &[
      ("baud_rate", node::IntAttribute),
      ("mode", node::StrAttribute),
      ("tx", node::RefAttribute),
      ("rx", node::RefAttribute)]) {
    return
  }

  let baud_rate: u32 = sub.get_int_attr("baud_rate").unwrap() as u32;
  let mode = sub.get_string_attr("mode").unwrap();

  let word_len = mode.as_str().chars().nth(0).unwrap().to_digit(10).unwrap() as u8;
  let parity = TokenString(
      match mode.as_str().chars().nth(1) {
        Some('N') => "Parity::Disabled",
        Some('O') => "Parity::Odd",
        Some('E') => "Parity::Even",
        Some('1') => "Parity::Forced1",
        Some('0') => "Parity::Forced0",
        _ => panic!(),
      }.to_string());
  let stop_bits = mode.as_str().chars().nth(2).unwrap().to_digit(10).unwrap() as u8;

  sub.set_type_name("zinc::hal::lpc17xx::uart::UART".to_string());
  let uart_name = TokenString(sub.name.clone().unwrap());

  let st = quote_stmt!(&*cx,
      let $uart_name = zinc::hal::lpc17xx::uart::UART::new(
          zinc::hal::lpc17xx::uart::$uart_peripheral,
          $baud_rate,
          $word_len,
          zinc::hal::uart::$parity,
          $stop_bits)
  ).unwrap();
  builder.add_main_statement(st);
}

pub fn build_uart_gpio(builder: &Builder, uart_idx: usize, name: &str,
    istx: bool) {
  let node = builder.pt().get_by_name(name).unwrap();
  let direction = (if istx {"out"} else {"in"}).to_string();
  let function = format!("{}{}", if istx {"txd"} else {"rxd"}, uart_idx);
  node.attributes.borrow_mut().insert("direction".to_string(),
        Rc::new(node::Attribute::new_nosp(node::StrValue(direction))));
  node.attributes.borrow_mut().insert("function".to_string(),
        Rc::new(node::Attribute::new_nosp(node::StrValue(function))));
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
  use builder::Builder;
  use test_helpers::{assert_equal_source, with_parsed};

  #[test]
  fn builds_uart() {
    with_parsed("
      mcu {
        clock {
          system_frequency = 100_000_000;
        }
      }
      uart {
        uart@0 {
          baud_rate = 9600;
          mode = \"8N1\";
          tx = &uart_tx;
          rx = &uart_rx;
        }
      }
      gpio {
        uart_tx@0;
        uart_rx@1;
      }
      ", |cx, failed, pt| {
      let mut builder = Builder::new(pt.clone(), cx);
      super::mutate_pins(&mut builder, cx, pt.get_by_name("uart").unwrap());
      super::build_uart(&mut builder, cx, pt.get_by_name("uart").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts().len() == 1);

      assert_equal_source(builder.main_stmts()[0].deref(),
          "let uart = zinc::hal::lpc17xx::uart::UART::new(
               zinc::hal::lpc17xx::uart::UARTPeripheral::UART0,
               9600u32,
               8u8,
               zinc::hal::uart::Parity::Disabled,
               1u8);");

      let tx_node = pt.get_by_name("uart_tx").unwrap();
      assert!(tx_node.get_string_attr("direction").unwrap() == "out".to_string());
      assert!(tx_node.get_string_attr("function").unwrap() == "txd0".to_string());

      let rx_node = pt.get_by_name("uart_rx").unwrap();
      assert!(rx_node.get_string_attr("direction").unwrap() == "in".to_string());
      assert!(rx_node.get_string_attr("function").unwrap() == "rxd0".to_string());
    });
  }
}
