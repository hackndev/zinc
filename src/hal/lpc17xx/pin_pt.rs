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

use builder::{Builder, TokenString};
use node;
use super::pinmap;

pub fn build_pin(builder: &mut Builder, cx: &mut ExtCtxt,
    node: Rc<node::Node>) {
  if !node.expect_no_attributes(cx) { return }

  for (port_path, port_node) in node.subnodes.iter() {
    if !port_node.expect_no_attributes(cx) { continue }

    let port_str = format!("Port{}", match from_str::<uint>(port_path.as_slice()).unwrap() {
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

      let direction_str = if pin_node.get_string_attr("function").is_some() {
        "core::option::None"
      } else {
        match pin_node.get_string_attr("direction").unwrap().as_slice() {
          "out" => "core::option::Some(zinc::hal::pin::Out)",
          "in"  => "core::option::Some(zinc::hal::pin::In)",
          other => {
            let attr = pin_node.get_attr("direction");
            cx.parse_sess().span_diagnostic.span_err(attr.value_span,
                format!("unknown direction `{}`, allowed values: `in`, `out`",
                    other).as_slice());
            continue;
          }
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

      pin_node.type_name.set(Some("zinc::hal::lpc17xx::pin::Pin"));

      let st = quote_stmt!(&*cx,
          let $pin_name = zinc::hal::lpc17xx::pin::Pin::new(
              zinc::hal::lpc17xx::pin::$port,
              $pin,
              zinc::hal::lpc17xx::pin::$function,
              $direction);
      );
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
      let mut builder = Builder::new(pt.clone());
      super::build_pin(&mut builder, cx, pt.get_by_path("gpio").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);

      assert_equal_source(builder.main_stmts.get(0),
          "let p1 = zinc::hal::lpc17xx::pin::Pin::new(
               zinc::hal::lpc17xx::pin::Port0,
               1u8,
               zinc::hal::lpc17xx::pin::GPIO,
               core::option::Some(zinc::hal::pin::In));");
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
      let mut builder = Builder::new(pt.clone());
      super::build_pin(&mut builder, cx, pt.get_by_path("gpio").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);

      assert_equal_source(builder.main_stmts.get(0),
          "let p2 = zinc::hal::lpc17xx::pin::Pin::new(
               zinc::hal::lpc17xx::pin::Port0,
               2u8,
               zinc::hal::lpc17xx::pin::GPIO,
               core::option::Some(zinc::hal::pin::Out));");
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
      let mut builder = Builder::new(pt.clone());
      super::build_pin(&mut builder, cx, pt.get_by_path("gpio").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);

      assert_equal_source(builder.main_stmts.get(0),
          "let p3 = zinc::hal::lpc17xx::pin::Pin::new(
               zinc::hal::lpc17xx::pin::Port0,
               3u8,
               zinc::hal::lpc17xx::pin::AltFunction2,
               core::option::None);");
    });
  }
}
