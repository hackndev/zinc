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

mod pinmap;
mod init_pt;

pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  node.get_by_path("clock").and_then(|sub| -> Option<bool> {
    init_pt::build_clock(builder, cx, sub);
    None
  });

  node.get_by_path("timer").and_then(|sub| -> Option<bool> {
    build_timer(builder, cx, sub);
    None
  });

  node.get_by_path("uart").and_then(|sub| -> Option<bool> {
    build_uart(builder, cx, sub);
    None
  });

  node.get_by_path("gpio").and_then(|sub| -> Option<bool> {
    build_gpio(builder, cx, sub);
    None
  });
}

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
      0..3 => TokenString(format!("timer::Timer{}", timer_index)),
      other => {
        cx.parse_sess().span_diagnostic.span_err(sub.path_span,
            format!("unknown timer index `{}`, allowed indexes: 0, 1, 2, 3",
                other).as_slice());
        continue;
      }
    };

    sub.type_name.set(Some("zinc::hal::lpc17xx::timer::Timer"));

    let st = quote_stmt!(&*cx,
        let $name = {
          use zinc::hal::lpc17xx::timer;
          let conf = timer::TimerConf {
            timer: $timer_name,
            counter: $counter,
            divisor: $divisor,
          };
          conf.setup()
        }
    );
    builder.add_main_statement(st);
  }
}

pub fn build_gpio(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) { return }

  for (port_path, port_node) in node.subnodes.iter() {
    if !port_node.expect_no_attributes(cx) { continue }

    let port_str = format!("pin::Port{}", match from_str::<uint>(port_path.as_slice()).unwrap() {
      0..4 => port_path,
      other => {
        cx.parse_sess().span_diagnostic.span_err(port_node.path_span,
            format!("unknown port `{}`, allowed values: 0..4",
                other).as_slice());
        continue;
      }
    });
    let port = TokenString(port_str);

    for (pin_path, pin_node) in port_node.subnodes.iter() {
      if pin_node.name.is_none() {
        cx.parse_sess().span_diagnostic.span_err(pin_node.name_span,
            "pin node must have a name");
        continue;
      }

      let direction_str = match pin_node.get_string_attr("direction")
          .unwrap().as_slice() {
        "out" => "hal::gpio::Out",
        "in"  => "hal::gpio::In",
        other => {
          let attr = pin_node.get_attr("direction");
          cx.parse_sess().span_diagnostic.span_err(attr.value_span,
              format!("unknown direction `{}`, allowed values: `in`, `out`",
                  other).as_slice());
          continue;
        }
      };
      let direction = TokenString(direction_str.to_str());

      let pin_str = match from_str::<uint>(pin_path.as_slice()).unwrap() {
        0..31 => pin_path,
        other => {
          cx.parse_sess().span_diagnostic.span_err(pin_node.path_span,
              format!("unknown pin `{}`, allowed values: 0..31",
                  other).as_slice());
          continue;
        }
      };

      let port_def = pinmap::port_def();
      let function_str = match pin_node.get_string_attr("function") {
        None => "GPIO".to_str(),
        Some(fun) => {
          let pins = port_def.get(port_path);
          let maybe_pin = pins.get(from_str(pin_path.as_slice()).unwrap());
          match maybe_pin {
            &None => {
              cx.parse_sess().span_diagnostic.span_err(
                  pin_node.get_attr("function").value_span,
                  format!("unknown pin function `{}`, only GPIO avaliable on this pin",
                      fun).as_slice());
              continue;
            }
            &Some(ref pin_funcs) => {
              let maybe_func = pin_funcs.find(&fun);
              match maybe_func {
                None => {
                  let avaliable: Vec<String> = pin_funcs.keys().map(|k|{k.to_str()}).collect();
                  cx.parse_sess().span_diagnostic.span_err(
                      pin_node.get_attr("function").value_span,
                      format!("unknown pin function `{}`, allowed functions: {}",
                          fun, avaliable.connect(", ")).as_slice());
                  continue;
                },
                Some(func_idx) => {
                  format!("AltFunction{}", func_idx)
                }
              }
            }
          }
        }
      };

      let function = TokenString(function_str);
      let pin = TokenString(format!("{}u8", pin_str));
      let pin_name = TokenString(pin_node.name.clone().unwrap());
      let pin_name_conf = TokenString(format!(
          "{}_conf", pin_node.name.clone().unwrap()));

      pin_node.type_name.set(Some("zinc::hal::lpc17xx::gpio::GPIO"));

      let st_conf = quote_stmt!(&*cx,
          let $pin_name_conf = {
            use zinc::hal;
            use zinc::hal::lpc17xx::{pin, gpio};
            let conf = gpio::GPIOConf {
              pin: pin::PinConf {
                port: $port,
                pin: $pin,
                function: pin::$function,
              },
              direction: $direction,
            };
            conf.pin.setup();
            conf
          }
      );
      let st = quote_stmt!(&*cx, let $pin_name = $pin_name_conf.setup() );
      builder.add_main_statement(st_conf);
      builder.add_main_statement(st);
    }
  }
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
