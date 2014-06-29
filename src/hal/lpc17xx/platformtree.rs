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

mod system_clock_pt;
mod timer_pt;
mod pin_pt;
mod pinmap;

pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  node.get_by_path("clock").and_then(|sub| -> Option<bool> {
    system_clock_pt::build_clock(builder, cx, sub);
    None
  });

  node.get_by_path("timer").and_then(|sub| -> Option<bool> {
    timer_pt::build_timer(builder, cx, sub);
    None
  });

  node.get_by_path("uart").and_then(|sub| -> Option<bool> {
    build_uart(builder, cx, sub);
    None
  });

  node.get_by_path("gpio").and_then(|sub| -> Option<bool> {
    pin_pt::build_pin(builder, cx, sub);
    None
  });
}

pub fn build_uart(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) { return }

  for (path, sub) in node.subnodes.iter() {
    let uart_peripheral_str = format!("UART{}",
        match from_str::<uint>(path.as_slice()).unwrap() {
          0|2|3 => path,
          other => {
            cx.parse_sess().span_diagnostic.span_err(sub.path_span,
                format!("unknown UART `{}`, allowed values: 0, 2, 3",
                    other).as_slice());
            continue;
          }
        });
    let uart_peripheral = TokenString(uart_peripheral_str);

    if sub.name.is_none() {
      cx.parse_sess().span_diagnostic.span_err(sub.name_span,
          "UART node must have a name");
      continue;
    }

    if !sub.expect_attributes(cx, [
        ("baud_rate", node::IntAttribute),
        ("mode", node::StrAttribute),
        ("tx", node::RefAttribute),
        ("rx", node::RefAttribute)]) {
      continue;
    }

    let baud_rate: u32 = sub.get_int_attr("baud_rate").unwrap() as u32;
    let mode = sub.get_string_attr("mode").unwrap();
    let tx_node_name = sub.get_ref_attr("tx").unwrap();
    let rx_node_name = sub.get_ref_attr("rx").unwrap();

    let word_len = mode.as_slice().char_at(0).to_digit(10).unwrap() as u8;
    let parity = TokenString(
        match mode.as_slice().char_at(1) {
          'N' => "Disabled",
          'O' => "Odd",
          'E' => "Even",
          '1' => "Forced1",
          '0' => "Forced0",
          _ => fail!(),
        }.to_str());
    let stop_bits = mode.as_slice().char_at(2).to_digit(10).unwrap() as u8;
    build_uart_gpio(builder, from_str(path.as_slice()).unwrap(),
        tx_node_name.as_slice(), true);
    build_uart_gpio(builder, from_str(path.as_slice()).unwrap(),
        rx_node_name.as_slice(), false);

    sub.type_name.set(Some("zinc::hal::lpc17xx::uart::UART"));
    let uart_name = TokenString(sub.name.clone().unwrap());
    let uart_name_conf = TokenString(format!("{}_conf", sub.name.clone().unwrap()));

    let st_conf = quote_stmt!(&*cx,
        let $uart_name_conf = {
          use zinc::hal;
          use zinc::hal::lpc17xx::uart;
          uart::UARTConf {
            peripheral: uart::$uart_peripheral,
            baudrate: $baud_rate,
            word_len: $word_len,
            parity: hal::uart::$parity,
            stop_bits: $stop_bits,
          }
        }
    );
    let st = quote_stmt!(&*cx, let $uart_name = $uart_name_conf.setup() );
    builder.add_main_statement(st_conf);
    builder.add_main_statement(st);
  }
}

pub fn build_uart_gpio(builder: &Builder, uart_idx: uint, name: &str,
    istx: bool) {
  let node = builder.pt.get_by_name(name).unwrap();
  let direction = (if istx {"out"} else {"in"}).to_str();
  let function = format!("{}{}", if istx {"txd"} else {"rxd"}, uart_idx);
  node.attributes.borrow_mut().insert("direction".to_str(),
        node::Attribute::new_nosp(node::StrValue(direction)));
  node.attributes.borrow_mut().insert("function".to_str(),
        node::Attribute::new_nosp(node::StrValue(function)));
}
