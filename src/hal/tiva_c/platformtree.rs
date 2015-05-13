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

use builder::{Builder, add_node_dependency};
use node;

mod clock_pt;
mod pin_pt;
mod timer_pt;
mod uart_pt;

pub fn attach(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  for sub in node.subnodes().iter() {
    add_node_dependency(&node, sub);

    match sub.path.as_slice() {
      "clock" => clock_pt::attach(builder, cx, sub.clone()),
      "gpio"  => pin_pt  ::attach(builder, cx, sub.clone()),
      "timer" => timer_pt::attach(builder, cx, sub.clone()),
      "uart"  => uart_pt ::attach(builder, cx, sub.clone()),
      _       => (),
    }
  }
}

fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
  node.expect_subnodes(cx, &["clock", "gpio", "timer", "uart"]);
}

pub fn add_node_dependency_on_clock(builder: &mut Builder,
    node: &Rc<node::Node>) {
  let mcu_node = builder.pt().get_by_path("mcu").unwrap();
  let clock_node = mcu_node.get_by_path("clock").unwrap();
  add_node_dependency(node, &clock_node);
}
