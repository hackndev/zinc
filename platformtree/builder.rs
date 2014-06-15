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
use syntax::ast;
use syntax::codemap::{ExpnInfo, NameAndSpan, MacroBang};
use syntax::ext::base::ExtCtxt;

use lpc17xx_pt;
use node;

pub struct Builder {
  pub main_stmts: Vec<Gc<ast::Stmt>>,
}

impl Builder {
  pub fn new() -> Builder {
    Builder {
      main_stmts: Vec::new(),
    }
  }

  pub fn add_main_statement(&mut self, stmt: Gc<ast::Stmt>) {
    self.main_stmts.push(stmt);
  }
}

pub fn build_platformtree(cx: &mut ExtCtxt, pt: &node::PlatformTree) -> Builder {
  let mut builder = Builder::new();

  if !pt.expect_subnodes(cx, ["mcu"]) {
    return builder;  // TODO(farcaller): report error?
  }

  match pt.get("mcu") {
    Some(node) => build_mcu(&mut builder, cx, node),
    None => (),  // TODO(farcaller): should it actaully fail?
  }

  builder
}

fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt, node: &Gc<node::Node>) {
  cx.bt_push(ExpnInfo {
    call_site: node.name_span,
    callee: NameAndSpan {
      name: "platformtree".to_str(),
      format: MacroBang,
      span: None,
    },
  });

  match node.path.as_slice() {
    "lpc17xx" => lpc17xx_pt::build_mcu(builder, cx, node),
    other => {
      cx.parse_sess().span_diagnostic.span_err(node.name_span,
          format!("unknown mcu `{}`", other).as_slice());
    },
  }

  cx.bt_pop();
}
