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
fn builds_clock_init() {
  with_parsed("
    clock {
      source = \"main-oscillator\";
      source_frequency = 12_000_000;
      pll {
        m = 50;
        n = 3;
        divisor = 4;
      }
    }", |cx, failed, pt| {
    let mut builder = Builder::new(pt);
    lpc17xx_pt::build_clock(&mut builder, cx, pt.get_by_path("clock").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 1);

    assert_equal_source(builder.main_stmts.get(0),
        "{
          use zinc::hal::lpc17xx::init;
          init::init_clock(
              &init::Clock {
                source: init::Main(12000000),
                pll: init::PLL0 {
                  enabled: true,
                  m: 50u8,
                  n: 3u8,
                  divisor: 4u8,
                },
              }
          );
        };");
  });
}

#[test]
fn clock_provides_out_frequency() {
  with_parsed("
    clock {
      source = \"main-oscillator\";
      source_frequency = 12_000_000;
      pll {
        m = 50;
        n = 3;
        divisor = 4;
      }
    }", |cx, _, pt| {
    let mut builder = Builder::new(pt);
    let node = pt.get_by_path("clock").unwrap();
    lpc17xx_pt::build_clock(&mut builder, cx, node);

    let out_freq = node.get_int_attr("system_frequency");
    assert!(out_freq.is_some());
    assert!(out_freq.unwrap() == 100_000_000);
  });
}

#[test]
fn fails_to_parse_bad_clock_conf() {
  fails_to_build("lpc17xx@mcu { clock {
    no_source = 1;
    source_frequency = 12_000_000;
  }}");
  fails_to_build("lpc17xx@mcu { clock {
    source = \"missing\";
    source_frequency = 12_000_000;
  }}");
}

#[test]
fn fails_to_parse_no_pll_clock() {
  fails_to_build("lpc17xx@mcu { clock {
    source = \"main-oscillator\";
    source_frequency = 12_000_000;
  }}");
}

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
    lpc17xx_pt::build_timer(&mut builder, cx, pt.get_by_path("timer").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 1);

    assert_equal_source(builder.main_stmts.get(0),
        "let tim = {
          use zinc::hal::lpc17xx::timer;
          let conf = timer::TimerConf {
            timer: timer::Timer1,
            counter: 25u32,
            divisor: 4u8,
          };
          conf.setup()
        };");
  });
}

#[test]
fn builds_input_gpio() {
  with_parsed("
    gpio {
      0 {
        p1@1 { direction = \"in\"; }
      }
    }", |cx, failed, pt| {
    let mut builder = Builder::new(pt);
    lpc17xx_pt::build_gpio(&mut builder, cx, pt.get_by_path("gpio").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 2);

    assert_equal_source(builder.main_stmts.get(0),
        "let p1_conf = {
          use zinc::hal;
          use zinc::hal::lpc17xx::{pin, gpio};
          let conf = gpio::GPIOConf {
            pin: pin::PinConf {
              port: pin::Port0,
              pin: 1u8,
              function: pin::GPIO,
            },
            direction: hal::gpio::In,
          };
          conf.pin.setup();
          conf
        };");
    assert_equal_source(builder.main_stmts.get(1),
        "let p1 = p1_conf.setup();");
  });
}

#[test]
fn builds_output_gpio() {
  with_parsed("
    gpio {
      0 {
        p2@2 { direction = \"out\"; }
      }
    }", |cx, failed, pt| {
    let mut builder = Builder::new(pt);
    lpc17xx_pt::build_gpio(&mut builder, cx, pt.get_by_path("gpio").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 2);

    assert_equal_source(builder.main_stmts.get(0),
        "let p2_conf = {
          use zinc::hal;
          use zinc::hal::lpc17xx::{pin, gpio};
          let conf = gpio::GPIOConf {
            pin: pin::PinConf {
              port: pin::Port0,
              pin: 2u8,
              function: pin::GPIO,
            },
            direction: hal::gpio::Out,
          };
          conf.pin.setup();
          conf
        };");
    assert_equal_source(builder.main_stmts.get(1),
        "let p2 = p2_conf.setup();");
  });
}

#[test]
fn builds_altfn_gpio() {
  with_parsed("
    gpio {
      0 {
        p3@3 { direction = \"out\"; function = \"ad0_6\"; }
      }
    }", |cx, failed, pt| {
    let mut builder = Builder::new(pt);
    lpc17xx_pt::build_gpio(&mut builder, cx, pt.get_by_path("gpio").unwrap());
    assert!(unsafe{*failed} == false);
    assert!(builder.main_stmts.len() == 2);

    assert_equal_source(builder.main_stmts.get(0),
        "let p3_conf = {
          use zinc::hal;
          use zinc::hal::lpc17xx::{pin, gpio};
          let conf = gpio::GPIOConf {
            pin: pin::PinConf {
              port: pin::Port0,
              pin: 3u8,
              function: pin::AltFunction2,
            },
            direction: hal::gpio::Out,
          };
          conf.pin.setup();
          conf
        };");
    assert_equal_source(builder.main_stmts.get(1),
        "let p3 = p3_conf.setup();");
  });
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
