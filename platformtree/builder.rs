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

use std::gc::{Gc, GC};
use syntax::abi;
use syntax::ast;
use syntax::ast_util::empty_generics;
use syntax::codemap::{Span, mk_sp, BytePos, ExpnInfo, NameAndSpan, MacroBang};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::parse::token::InternedString;

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

  fn emit_main(&self, cx: &ExtCtxt) -> Gc<ast::Item> {
    // init stack
    let init_stack_stmt = cx.stmt_expr(quote_expr!(&*cx,
        zinc::hal::mem_init::init_stack();
    ));

    // init data
    let init_data_stmt = cx.stmt_expr(quote_expr!(&*cx,
        zinc::hal::mem_init::init_data();
    ));

    let mut stmts = vec!(init_stack_stmt, init_data_stmt);
    stmts = stmts.append(self.main_stmts.as_slice());

    let empty_span = mk_sp(BytePos(0), BytePos(0)); // TODO(farcaller): fix span
    let body = cx.block(empty_span, stmts, None);

    self.item_fn(cx, empty_span, "main", body)
  }

  // TODO(farcaller): emit based on sched.
  fn emit_morestack(&self, cx: &ExtCtxt) -> Gc<ast::Item> {
    let stmt = cx.stmt_expr(quote_expr!(&*cx,
        zinc::os::task::morestack();
        // or
        // core::intrinsics::abort()
    ));
    let empty_span = mk_sp(BytePos(0), BytePos(0));
    let body = cx.block(empty_span, vec!(stmt), None);
    self.item_fn(cx, empty_span, "__morestack", body)
  }

  // TODO(farcaller): emit based on sched.
  fn emit_sched(&self, cx: &ExtCtxt) -> Gc<ast::Item> {
    let stmt = cx.stmt_expr(quote_expr!(&*cx,
        zinc::os::task::task_scheduler();
    ));
    let empty_span = mk_sp(BytePos(0), BytePos(0));
    let body = cx.block(empty_span, vec!(stmt), None);
    self.item_fn(cx, empty_span, "task_scheduler", body)
  }

  pub fn emit_items(&self, cx: &ExtCtxt) -> Vec<Gc<ast::Item>> {
    vec!(self.emit_main(cx), self.emit_morestack(cx), self.emit_sched(cx))
  }

  fn item_fn(&self, cx: &ExtCtxt, span: Span, name: &str,
      body: ast::P<ast::Block>) -> Gc<ast::Item> {
    let attr_no_mangle = cx.attribute(span, cx.meta_word(
        span, InternedString::new("no_mangle")));
    let attr_no_split_stack = cx.attribute(span, cx.meta_word(
        span, InternedString::new("no_split_stack")));

    box(GC) ast::Item {
      ident: cx.ident_of(name),
      attrs: vec!(attr_no_mangle, attr_no_split_stack),
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemFn(
          cx.fn_decl(Vec::new(), cx.ty_nil()),
          ast::UnsafeFn,
          abi::Rust, // TODO(farcaller): should this be abi::C?
          empty_generics(),
          body),
      vis: ast::Public,
      span: span,
    }
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
