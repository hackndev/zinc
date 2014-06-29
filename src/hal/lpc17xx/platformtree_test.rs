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

use builder::Builder;
use lpc17xx_pt;
use test_helpers::{assert_equal_source, with_parsed, fails_to_build};

#[test]
fn fails_to_parse_garbage_attrs() {
  fails_to_build("lpc17xx@mcu { key = 1; }");
}

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
    let mut builder = Builder::new(pt);
    lpc17xx_pt::build_uart(&mut builder, cx, pt.get_by_path("uart").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 2);

    assert_equal_source(builder.main_stmts.get(0),
        "let uart_conf = {
          use zinc::hal;
          use zinc::hal::lpc17xx::uart;
          uart::UARTConf {
            peripheral: uart::UART0,
            baudrate: 9600u32,
            word_len: 8u8,
            parity: hal::uart::Disabled,
            stop_bits: 1u8,
          }
        };");
    assert_equal_source(builder.main_stmts.get(1),
        "let uart = uart_conf.setup();");

    let tx_node = pt.get_by_name("uart_tx").unwrap();
    assert!(tx_node.get_string_attr("direction").unwrap() == "out".to_str());
    assert!(tx_node.get_string_attr("function").unwrap() == "txd0".to_str());

    let rx_node = pt.get_by_name("uart_rx").unwrap();
    assert!(rx_node.get_string_attr("direction").unwrap() == "in".to_str());
    assert!(rx_node.get_string_attr("function").unwrap() == "rxd0".to_str());
  });
}
