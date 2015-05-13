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

use lpc17xx_pt;
// use tiva_c_pt;
use node;

use super::Builder;

pub fn attach(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  match node.name {
    Some(ref name) => {
      match name.as_slice() {
        "lpc17xx" => lpc17xx_pt::attach(builder, cx, node.clone()),
        // "tiva_c"  => tiva_c_pt::attach(builder, cx, node.clone()),
        _ => node.materializer.set(Some(fail_build_mcu as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>))),
      }
    },
    None => node.materializer.set(Some(fail_build_mcu as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>))),
  }
}

pub fn fail_build_mcu(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  match node.name {
    Some(ref name) => cx.parse_sess().span_diagnostic.span_err(
        node.name_span, format!("unknown mcu `{}`", name).as_slice()),
    None => cx.parse_sess().span_diagnostic.span_err(
        node.name_span, "`mcu` node must have a name"),
  }
}
