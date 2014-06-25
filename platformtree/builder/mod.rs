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
use syntax::ast::TokenTree;
use syntax::ast;
use syntax::ast_util::empty_generics;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::parse::token::InternedString;

use node;

mod mcu;
mod os;

pub struct Builder {
  pub main_stmts: Vec<Gc<ast::Stmt>>,
  pub type_items: Vec<Gc<ast::Item>>,
  pub pt: Gc<node::PlatformTree>,
}

impl Builder {
  pub fn new(pt: &Gc<node::PlatformTree>) -> Builder {
    Builder {
      main_stmts: Vec::new(),
      type_items: Vec::new(),
      pt: *pt,
    }
  }

  pub fn add_main_statement(&mut self, stmt: Gc<ast::Stmt>) {
    self.main_stmts.push(stmt);
  }

  pub fn add_type_item(&mut self, item: Gc<ast::Item>) {
    self.type_items.push(item);
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

    let empty_span = DUMMY_SP; // TODO(farcaller): fix span
    let body = cx.block(empty_span, stmts, None);

    self.item_fn(cx, empty_span, "main", body)
  }

  fn emit_morestack(&self, cx: &ExtCtxt) -> Gc<ast::Item> {
    let stmt = cx.stmt_expr(quote_expr!(&*cx,
        core::intrinsics::abort()
        // or
        // zinc::os::task::morestack();
    ));
    let empty_span = DUMMY_SP;
    let body = cx.block(empty_span, vec!(stmt), None);
    self.item_fn(cx, empty_span, "__morestack", body)
  }

  pub fn emit_items(&self, cx: &ExtCtxt) -> Vec<Gc<ast::Item>> {
    let non_camel_case_types = cx.meta_word(DUMMY_SP,
        InternedString::new("non_camel_case_types"));
    let allow = cx.meta_list(
        DUMMY_SP,
        InternedString::new("allow"), vec!(non_camel_case_types));
    let allow_noncamel = cx.attribute(DUMMY_SP, allow);
    let use_zinc = cx.view_use_simple(DUMMY_SP, ast::Inherited, cx.path_ident(
        DUMMY_SP, cx.ident_of("zinc")));
    let pt_mod_item = cx.item_mod(DUMMY_SP, DUMMY_SP, cx.ident_of("pt"),
        vec!(allow_noncamel), vec!(use_zinc), self.type_items.clone());

    vec!(pt_mod_item, self.emit_main(cx), self.emit_morestack(cx))
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

pub struct TokenString {
  pub s: String,
}

impl TokenString {
  pub fn new(s: String) -> TokenString {
    TokenString {
      s: s,
    }
  }
}

impl ToTokens for TokenString {
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
    (cx as &ExtParseUtils).parse_tts(self.s.clone())
  }
}

pub fn build_platformtree(cx: &mut ExtCtxt, pt: &Gc<node::PlatformTree>) -> Builder {
  let mut builder = Builder::new(pt);

  if !pt.expect_subnodes(cx, ["mcu", "os"]) {
    return builder;  // TODO(farcaller): report error?
  }

  match pt.get_by_path("mcu") {
    Some(node) => mcu::build_mcu(&mut builder, cx, node),
    None => (),  // TODO(farcaller): should it actaully fail?
  }

  match pt.get_by_path("os") {
    Some(node) => os::build_os(&mut builder, cx, node),
    None => {
      // TODO(farcaller): provide span for whole PT?
      cx.parse_sess().span_diagnostic.span_err(DUMMY_SP,
          "root node `os` must be present");
    }
  }

  builder
}

#[cfg(test)]
mod test {
  use test_helpers::fails_to_build;

  #[test]
  fn fails_to_parse_pt_with_unknown_root_node() {
    fails_to_build("unknown@node {}");
  }

  #[test]
  fn fails_to_parse_pt_with_unknown_mcu() {
    fails_to_build("mcu@bad {}");
  }
}
