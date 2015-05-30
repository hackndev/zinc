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

mod dht22_pt;

pub fn attach(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  for sub in node.subnodes().iter() {
    add_node_dependency(&node, sub);

    match sub.path.as_str() {
      "dht22" => dht22_pt::attach(builder, cx, sub.clone()),
      _ => (),
    }
  }
}

fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
  node.expect_subnodes(cx, &["dht22"]);
}
