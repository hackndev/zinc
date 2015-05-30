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

use builder::{Builder, add_node_dependency};
use node;

mod system_clock_pt;
mod timer_pt;
mod pin_pt;
mod uart_pt;

mod pinmap;

pub fn attach(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  for sub in node.subnodes().iter() {
    add_node_dependency(&node, sub);

    match sub.path.as_str() {
      "clock" => system_clock_pt::attach(builder, cx, sub.clone()),
      "timer" => timer_pt::attach(builder, cx, sub.clone()),
      "uart"  => uart_pt::attach(builder, cx, sub.clone()),
      "gpio"  => pin_pt::attach(builder, cx, sub.clone()),
      _ => (),
    }
  }
}

fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
  node.expect_subnodes(cx, &["clock", "timer", "uart", "gpio"]);
}

pub fn add_node_dependency_on_clock(builder: &mut Builder,
    node: &Rc<node::Node>) {
  let mcu_node = builder.pt().get_by_path("mcu").unwrap();
  let clock_node = mcu_node.get_by_path("clock").unwrap();
  add_node_dependency(node, &clock_node);
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
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
      let items = Builder::build(cx, pt)
        .expect(format!("Unexpected failure on {}", line!()).as_str())
        .emit_items(cx);

      assert!(unsafe{*failed} == false);
      assert!(items.len() == 4);

      assert_equal_items(items[1].deref(), "
          #[no_mangle]
          #[allow(unused_variables)]
          pub unsafe fn platformtree_main() -> () {
            zinc::hal::mem_init::init_stack();
            zinc::hal::mem_init::init_data();
            {
                use zinc::hal::lpc17xx::system_clock;
                system_clock::init_clock(&system_clock::Clock{
                  source: system_clock::ClockSource::Main(12000000),
                  pll: core::option::Option::Some(system_clock::PLL0{
                    m: 50u8,
                    n: 3u8,
                    divisor: 4u8,
                  }),
                });
            };
            let timer = zinc::hal::lpc17xx::timer::Timer::new(
                zinc::hal::lpc17xx::timer::TimerPeripheral::Timer1, 25u32, 4u8);
            let uart_tx = zinc::hal::lpc17xx::pin::Pin::new(
                zinc::hal::lpc17xx::pin::Port::Port0,
                2u8,
                zinc::hal::lpc17xx::pin::Function::AltFunction1,
                core::option::Option::None);
            let uart_rx = zinc::hal::lpc17xx::pin::Pin::new(
                zinc::hal::lpc17xx::pin::Port::Port0,
                3u8,
                zinc::hal::lpc17xx::pin::Function::AltFunction1,
                core::option::Option::None);
            let uart = zinc::hal::lpc17xx::uart::UART::new(
                zinc::hal::lpc17xx::uart::UARTPeripheral::UART0,
                115200u32,
                8u8,
                zinc::hal::uart::Parity::Disabled,
                1u8);
            let led4 = zinc::hal::lpc17xx::pin::Pin::new(
                zinc::hal::lpc17xx::pin::Port::Port1,
                23u8,
                zinc::hal::lpc17xx::pin::Function::Gpio,
                core::option::Option::Some(zinc::hal::pin::Out));
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
