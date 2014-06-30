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

use builder::Builder;
use node;

mod system_clock_pt;
mod timer_pt;
mod pin_pt;
mod uart_pt;

mod pinmap;

pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
    node: Rc<node::Node>) {
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
    uart_pt::build_uart(builder, cx, sub);
    None
  });

  node.get_by_path("gpio").and_then(|sub| -> Option<bool> {
    pin_pt::build_pin(builder, cx, sub);
    None
  });
}

#[cfg(test)]
mod test {
  use builder::Builder;
  use test_helpers::{assert_equal_items, with_parsed, fails_to_build};

  #[test]
  fn fails_to_parse_garbage_attrs() {
    fails_to_build("lpc17xx@mcu { key = 1; }");
  }

  #[test]
  fn builds_lpc17xx_pt() {
    with_parsed("
      lpc17xx@mcu {
        clock {
          source = \"main-oscillator\";
          source_frequency = 12_000_000;
          pll {
            m = 50;
            n = 3;
            divisor = 4;
          }
        }

        timer {
          timer@1 {
            counter = 25;
            divisor = 4;
          }
        }

        uart {
          uart@0 {
            baud_rate = 115200;
            mode = \"8N1\";
            tx = &uart_tx;
            rx = &uart_rx;
          }
        }

        gpio {
          0 {
            uart_tx@2;
            uart_rx@3;
          }
          1 {
            led4@23 { direction = \"out\"; }
          }
        }
      }

      os {
        single_task {
          loop = \"run\";
          args {
            timer = &timer;
            txled = &led4;
            uart = &uart;
          }
        }
      }", |cx, failed, pt| {
      let builder = Builder::build(cx, pt);
      let items = builder.emit_items(cx);
      assert!(unsafe{*failed} == false);

      let items = builder.emit_items(cx);
      assert!(items.len() == 3);

      assert_equal_items(items.get(1), "
          #[no_mangle]
          #[no_split_stack]
          #[allow(unused_variable)]
          pub unsafe fn main() {
            zinc::hal::mem_init::init_stack();
            zinc::hal::mem_init::init_data();
            {
                use zinc::hal::lpc17xx::system_clock;
                system_clock::init_clock(&system_clock::Clock{
                  source: system_clock::Main(12000000),
                  pll: core::option::Some(system_clock::PLL0{
                    m: 50u8,
                    n: 3u8,
                    divisor: 4u8,
                  }),
                });
            };
            let timer = zinc::hal::lpc17xx::timer::Timer::new(
                zinc::hal::lpc17xx::timer::Timer1, 25u32, 4u8);
            let uart = zinc::hal::lpc17xx::uart::UART::new(
                zinc::hal::lpc17xx::uart::UART0,
                115200u32,
                8u8,
                zinc::hal::uart::Disabled,
                1u8);
            let led4 = zinc::hal::lpc17xx::pin::Pin::new(
                zinc::hal::lpc17xx::pin::Port1,
                23u8,
                zinc::hal::lpc17xx::pin::GPIO,
                core::option::Some(zinc::hal::pin::Out));
            let uart_tx = zinc::hal::lpc17xx::pin::Pin::new(
                zinc::hal::lpc17xx::pin::Port0,
                2u8,
                zinc::hal::lpc17xx::pin::AltFunction1,
                core::option::None);
            let uart_rx = zinc::hal::lpc17xx::pin::Pin::new(
                zinc::hal::lpc17xx::pin::Port0,
                3u8,
                zinc::hal::lpc17xx::pin::AltFunction1,
                core::option::None);
            loop {
              run(&pt::run_args{
                timer: &timer,
                txled: &led4,
                uart: &uart,
              });
            }
          }");
    });
  }
}
