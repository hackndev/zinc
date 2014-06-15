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
use syntax::ext::base::ExtCtxt;
use syntax::codemap::{ExpnInfo, NameAndSpan, MacroBang};

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

  for n in pt.iter() {
    match n.name {
      None => {
        cx.parse_sess().span_diagnostic.span_err(n.name_span,
            "root node cannot be anonymous");
        continue;
      },
      Some(ref name) => {
        match name.as_slice() {
          "mcu" => {
            build_mcu(&mut builder, cx, n);
          },
          other => {
            cx.parse_sess().span_diagnostic.span_err(n.name_span,
                format!("unknown root node `{}`", other).as_slice());
            continue;
          }
        }
      },
    }
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

mod lpc17xx_pt {
  use std::gc::Gc;
  use syntax::ext::base::ExtCtxt;
  use syntax::ext::build::AstBuilder;

  use super::Builder;
  use node;


  pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt, _: &Gc<node::Node>) {
    // init stack
    builder.add_main_statement(cx.stmt_expr(quote_expr!(&*cx,
        {
          use zinc::hal::stack;
          extern { static _eglobals: u32; }
          stack::set_stack_limit((&_eglobals as *u32) as u32);
        }
    )));

    // init data
    builder.add_main_statement(cx.stmt_expr(quote_expr!(&*cx,
        zinc::hal::mem_init::init_data();
    )));
  }
}
