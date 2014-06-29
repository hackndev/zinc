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
use syntax::ext::build::AstBuilder;

use builder::{Builder, TokenString};
use node;

mod dht22_pt;

pub fn build_drivers(builder: &mut Builder, cx: &mut ExtCtxt, node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  for (path, sub) in node.subnodes.iter() {
    match path.as_slice() {
      "dht22" => dht22_pt::build_dht22(builder, cx, sub),
      other => {
        cx.span_err(
            sub.path_span,
            format!("unknown driver `{}`", other).as_slice());
      }
    }
  }
}
