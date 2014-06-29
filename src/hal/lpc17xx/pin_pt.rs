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
use super::pinmap;

pub fn build_pin(builder: &mut Builder, cx: &mut ExtCtxt,
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

#[cfg(test)]
mod test {
  use builder::Builder;
  use test_helpers::{assert_equal_source, with_parsed};

  #[test]
  fn builds_input_gpio() {
    with_parsed("
      gpio {
        0 {
          p1@1 { direction = \"in\"; }
        }
      }", |cx, failed, pt| {
      let mut builder = Builder::new(pt);
      super::build_pin(&mut builder, cx, pt.get_by_path("gpio").unwrap());
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
      super::build_pin(&mut builder, cx, pt.get_by_path("gpio").unwrap());
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
      super::build_pin(&mut builder, cx, pt.get_by_path("gpio").unwrap());
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
}
