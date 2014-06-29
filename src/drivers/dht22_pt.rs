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

pub fn build_dht22(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  if !node.expect_no_subnodes(cx) {
    return;
  }

  if !node.expect_attributes(cx,
      [("pin", node::RefAttribute), ("timer", node::RefAttribute)]) {
    return;
  }

  let pin_node_name = node.get_ref_attr("pin").unwrap();
  let timer_node_name = node.get_ref_attr("timer").unwrap();
}
