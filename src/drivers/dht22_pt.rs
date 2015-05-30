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

pub fn attach(builder: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(build_dht22 as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  node.mutator.set(Some(mutate_pin as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));

  let pin_node_name = node.get_ref_attr("pin").unwrap();
  let pin_node = builder.pt().get_by_name(pin_node_name.as_str()).unwrap();
  add_node_dependency(&node, &pin_node);

  let timer_node_name = node.get_ref_attr("timer").unwrap();
  let timer_node = builder.pt().get_by_name(timer_node_name.as_str()).unwrap();
  add_node_dependency(&node, &timer_node);
}

fn mutate_pin(builder: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  let pin_node_name = node.get_ref_attr("pin").unwrap();
  let pin_node = builder.pt().get_by_name(pin_node_name.as_str()).unwrap();
  pin_node.attributes.borrow_mut().insert("direction".to_string(),
        Rc::new(node::Attribute::new_nosp(node::StrValue("out".to_string()))));
}

fn build_dht22(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  if !node.expect_no_subnodes(cx) {return}

  if !node.expect_attributes(cx,
      &[("pin", node::RefAttribute), ("timer", node::RefAttribute)]) {
    return
  }

  let pin_node_name = node.get_ref_attr("pin").unwrap();
  let timer_node_name = node.get_ref_attr("timer").unwrap();

  let pin = TokenString(pin_node_name);
  let timer = TokenString(timer_node_name);
  let name = TokenString(node.name.clone().unwrap());

  let typename = format!("zinc::drivers::dht22::DHT22");
  node.set_type_name(typename);
  let ty_params = vec!(
      "'a".to_string(),
      "zinc::hal::timer::Timer".to_string(),
      "zinc::hal::pin::Gpio".to_string());
  node.set_type_params(ty_params);

  let st = quote_stmt!(&*cx,
      let $name = zinc::drivers::dht22::DHT22::new(&$timer, &$pin);
  ).unwrap();
  builder.add_main_statement(st);
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
  use builder::Builder;
  use test_helpers::{assert_equal_source, with_parsed};
  use hamcrest::{assert_that, is, equal_to};

  #[test]
  fn builds_lpc17xx_pt() {
    with_parsed("
      timer@timer;
      pin@pin;
      dht@dht22 {
        pin = &pin;
        timer = &timer;
      }", |cx, failed, pt| {
      let mut builder = Builder::new(pt.clone(), cx);
      pt.get_by_name("timer").unwrap().set_type_name("T".to_string());
      pt.get_by_name("pin").unwrap().set_type_name("P".to_string());
      super::mutate_pin(&mut builder, cx, pt.get_by_name("dht").unwrap());
      super::build_dht22(&mut builder, cx, pt.get_by_name("dht").unwrap());
      assert_that(unsafe{*failed}, is(equal_to(false)));
      assert_that(builder.main_stmts().len(), is(equal_to(1usize)));

      assert_equal_source(builder.main_stmts()[0].deref(),
          "let dht = zinc::drivers::dht22::DHT22::new(&timer, &pin);");

      let pin_node = pt.get_by_name("pin").unwrap();
      assert_that(pin_node.get_string_attr("direction").unwrap(),
          is(equal_to("out".to_string())));
    });
  }
}
