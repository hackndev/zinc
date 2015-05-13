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
use super::pinmap;

pub fn attach(builder: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  for port_node in node.subnodes().iter() {
    port_node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
    add_node_dependency(&node, port_node);
    for pin_node in port_node.subnodes().iter() {
      pin_node.materializer.set(Some(build_pin as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
      add_node_dependency(port_node, pin_node);
      super::add_node_dependency_on_clock(builder, pin_node);
    }
  }
}

pub fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
}

fn build_pin(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  let port_node = node.parent.clone().unwrap().upgrade().unwrap();
  let ref port_path = port_node.path;
  let port_str = format!("Port{}", match port_path.as_slice().parse::<usize>().unwrap() {
    0...4 => port_path,
    other => {
      cx.parse_sess().span_diagnostic.span_err(port_node.path_span,
          format!("unknown port `{}`, allowed values: 0...4",
              other).as_slice());
      return;
    }
  });
  let port = TokenString(port_str);

  if node.name.is_none() {
    cx.parse_sess().span_diagnostic.span_err(node.name_span,
        "pin node must have a name");
    return;
  }

  let direction_str = if node.get_string_attr("function").is_some() {
    "core::option::Option::None"
  } else {
    match node.get_string_attr("direction").unwrap().as_slice() {
      "out" => "core::option::Option::Some(zinc::hal::pin::Out)",
      "in"  => "core::option::Option::Some(zinc::hal::pin::In)",
      other => {
        let attr = node.get_attr("direction");
        cx.parse_sess().span_diagnostic.span_err(attr.value_span,
            format!("unknown direction `{}`, allowed values: `in`, `out`",
                other).as_slice());
        return;
      }
    }
  };
  let direction = TokenString(direction_str.to_string());

  let pin_str = match node.path.as_slice().parse::<usize>().unwrap() {
    0...31 => &node.path,
    other  => {
      cx.parse_sess().span_diagnostic.span_err(node.path_span,
          format!("unknown pin `{}`, allowed values: 0...31",
              other).as_slice());
      return;
    }
  };

  let port_def = pinmap::port_def();
  let function_str = match node.get_string_attr("function") {
    None => "Gpio".to_string(),
    Some(fun) => {
      let pins = &port_def[*port_path];
      let maybe_pin_index: usize = node.path.as_slice().parse().unwrap();
      let maybe_pin: &Option<pinmap::PinDef> = pins.get(maybe_pin_index).unwrap();
      match maybe_pin {
        &None => {
          cx.parse_sess().span_diagnostic.span_err(
              node.get_attr("function").value_span,
              format!("unknown pin function `{}`, only GPIO avaliable on this pin",
                  fun).as_slice());
          return;
        }
        &Some(ref pin_funcs) => {
          let maybe_func = pin_funcs.get(&fun);
          match maybe_func {
            None => {
              let avaliable: Vec<String> = pin_funcs.keys().map(|k|{k.to_string()}).collect();
              cx.parse_sess().span_diagnostic.span_err(
                  node.get_attr("function").value_span,
                  format!("unknown pin function `{}`, allowed functions: {}",
                      fun, avaliable.connect(", ")).as_slice());
              return;
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
  let pin_name = TokenString(node.name.clone().unwrap());

  node.set_type_name("zinc::hal::lpc17xx::pin::Pin".to_string());

  let st = quote_stmt!(&*cx,
      let $pin_name = zinc::hal::lpc17xx::pin::Pin::new(
          zinc::hal::lpc17xx::pin::Port::$port,
          $pin,
          zinc::hal::lpc17xx::pin::Function::$function,
          $direction);
  );
  builder.add_main_statement(st);
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
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
      let mut builder = Builder::new(pt.clone(), cx);
      super::build_pin(&mut builder, cx, pt.get_by_name("p1").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts().len() == 1);

      assert_equal_source(builder.main_stmts()[0].deref(),
          "let p1 = zinc::hal::lpc17xx::pin::Pin::new(
               zinc::hal::lpc17xx::pin::Port::Port0,
               1u8,
               zinc::hal::lpc17xx::pin::Function::Gpio,
               core::option::Option::Some(zinc::hal::pin::In));");
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
      let mut builder = Builder::new(pt.clone(), cx);
      super::build_pin(&mut builder, cx, pt.get_by_name("p2").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts().len() == 1);

      assert_equal_source(builder.main_stmts()[0].deref(),
          "let p2 = zinc::hal::lpc17xx::pin::Pin::new(
               zinc::hal::lpc17xx::pin::Port::Port0,
               2u8,
               zinc::hal::lpc17xx::pin::Function::Gpio,
               core::option::Option::Some(zinc::hal::pin::Out));");
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
      let mut builder = Builder::new(pt.clone(), cx);
      super::build_pin(&mut builder, cx, pt.get_by_name("p3").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts().len() == 1);

      assert_equal_source(builder.main_stmts()[0].deref(),
          "let p3 = zinc::hal::lpc17xx::pin::Pin::new(
               zinc::hal::lpc17xx::pin::Port::Port0,
               3u8,
               zinc::hal::lpc17xx::pin::Function::AltFunction2,
               core::option::Option::None);");
    });
  }
}
