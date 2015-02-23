// Zinc, the bare metal stack for rust.
// Copyright 2014 Lionel Flandrin <lionel@svkt.org>
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

fn get_port_id(s: &str) -> Option<char> {
    match s.len() {
        1 => match s.chars().nth(0).unwrap().to_uppercase() {
            p @ 'A'...'F' => Some(p),
            _             => None,
        },
        _ => None,
    }
}

fn build_pin(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  let port_node = node.parent.clone().unwrap().upgrade().unwrap();
  let ref port_path = port_node.path;

  let error = |&: err: &str | {
    cx.parse_sess().span_diagnostic.span_err(port_node.path_span, err);
  };

  let port_str = format!("Port{}", match get_port_id(port_path.as_slice()) {
    Some(port) => port,
    None => {
      cx.parse_sess().span_diagnostic.span_err(port_node.path_span,
          format!("unknown port `{}`, allowed values: a...f",
                  port_path).as_slice());
      return;
    }
  });

  let port = TokenString(port_str);

  if node.name.is_none() {
    error("pin node must have a name");
    return;
  }

  let direction_str =
    match node.get_string_attr("direction").unwrap().as_slice() {
      "out" => "zinc::hal::pin::Out",
      "in"  => "zinc::hal::pin::In",
      bad   => {
        error(format!("unknown direction `{}`, allowed values: `in`, `out`",
                      bad).as_slice());
        return;
      }
    };

  let direction = TokenString(direction_str.to_string());

  let function = match node.get_int_attr("function") {
    None       => 0, // Default to GPIO function
    Some(f)    => f as u8,
  };

  let pin_str = match node.path.as_slice().parse::<usize>().unwrap() {
    0 ...7  => &node.path,
    other  => {
      error(format!("unknown pin `{}`, allowed values: 0...7",
                    other).as_slice());
      return;
    }
  };

  let pin = TokenString(format!("{}u8", pin_str));
  let pin_name = TokenString(node.name.clone().unwrap());

  node.set_type_name("zinc::hal::tiva_c::pin::Pin".to_string());

  // TODO(simias): need to handle pin muxing
  let st = quote_stmt!(&*cx,
      let $pin_name = zinc::hal::tiva_c::pin::Pin::new(
          zinc::hal::tiva_c::pin::PortId::$port,
          $pin,
          $direction,
          $function);
  );
  builder.add_main_statement(st);
}
