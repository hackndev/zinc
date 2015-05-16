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
use std::ops::DerefMut;
use syntax::abi;
use syntax::ast::TokenTree;
use syntax::ast;
use syntax::ast_util::empty_generics;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::parse::token::InternedString;
use syntax::ptr::P;

use node;

mod mcu;
mod os;
pub mod meta_args;

pub struct Builder {
  main_stmts: Vec<P<ast::Stmt>>,
  type_items: Vec<P<ast::Item>>,
  pt: Rc<node::PlatformTree>,
}

impl Builder {
  pub fn build(cx: &mut ExtCtxt, pt: Rc<node::PlatformTree>) -> Option<Builder> {
    let mut builder = Builder::new(pt.clone(), cx);

    if !pt.expect_subnodes(cx, &["mcu", "os", "drivers"]) {
      return None;
    }

    match pt.get_by_path("mcu") {
      Some(node) => mcu::attach(&mut builder, cx, node),
      None => (),  // TODO(farcaller): should it actaully fail?
    }

    match pt.get_by_path("os") {
      Some(node) => os::attach(&mut builder, cx, node),
      None => (),  // TODO(farcaller): this should fail.
    }

    match pt.get_by_path("drivers") {
      Some(node) => ::drivers_pt::attach(&mut builder, cx, node),
      None => (),
    }

    for sub in pt.nodes().iter() {
      Builder::walk_mutate(&mut builder, cx, sub);
    }

    let base_node = pt.get_by_path("mcu").and_then(|mcu|{mcu.get_by_path("clock")});
    match base_node {
      Some(node) => Builder::walk_materialize(&mut builder, cx, node),
      None => {
        cx.parse_sess().span_diagnostic.span_err(DUMMY_SP,
            "root node `mcu::clock` must be present");
      }
    }

    Some(builder)
  }

  fn walk_mutate(builder: &mut Builder, cx: &mut ExtCtxt, node: &Rc<node::Node>) {
    let maybe_mut = node.mutator.get();
    if maybe_mut.is_some() {
      maybe_mut.unwrap()(builder, cx, node.clone());
    }
    for sub in node.subnodes().iter() {
      Builder::walk_mutate(builder, cx, sub);
    }
  }

  // FIXME(farcaller): verify that all nodes have been materialized
  fn walk_materialize(builder: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
    let maybe_mat = node.materializer.get();
    if maybe_mat.is_some() {
      maybe_mat.unwrap()(builder, cx, node.clone());
    }
    let rev_depends = node.rev_depends_on.borrow();
    for weak_sub in rev_depends.iter() {
      let sub = weak_sub.upgrade().unwrap();
      let mut sub_deps = sub.depends_on.borrow_mut();
      let deps = sub_deps.deref_mut();
      let mut index = None;
      let mut i = 0usize;
      // FIXME: iter().position()
      for dep in deps.iter() {
        let strong_dep = dep.upgrade().unwrap();
        if node == strong_dep {
          index = Some(i);
          break;
        }
        i = i + 1;
      }
      if index.is_none() {
        panic!("no index found");
      } else {
        deps.remove(index.unwrap());
        if deps.len() == 0 {
          Builder::walk_materialize(builder, cx, sub.clone());
        }
      }
    }
  }

  pub fn new(pt: Rc<node::PlatformTree>, cx: &ExtCtxt) -> Builder {
    let use_zinc = cx.item_use_simple(DUMMY_SP, ast::Inherited, cx.path_ident(
        DUMMY_SP, cx.ident_of("zinc")));

    Builder {
      main_stmts: vec!(),
      type_items: vec!(use_zinc),
      pt: pt,
    }
  }

  pub fn main_stmts(&self) -> Vec<P<ast::Stmt>> {
    self.main_stmts.clone()
  }

  pub fn pt(&self) -> Rc<node::PlatformTree> {
    self.pt.clone()
  }

  pub fn add_main_statement(&mut self, stmt: P<ast::Stmt>) {
    self.main_stmts.push(stmt);
  }

  pub fn add_type_item(&mut self, item: P<ast::Item>) {
    self.type_items.push(item);
  }

  fn emit_main(&self, cx: &ExtCtxt) -> P<ast::Item> {
    // init stack
    let init_stack_stmt = cx.stmt_expr(quote_expr!(&*cx,
        zinc::hal::mem_init::init_stack();
    ));

    // init data
    let init_data_stmt = cx.stmt_expr(quote_expr!(&*cx,
        zinc::hal::mem_init::init_data();
    ));

    let mut stmts = vec!(init_stack_stmt, init_data_stmt);
    stmts.push_all(self.main_stmts.as_slice());

    let body = cx.block(DUMMY_SP, stmts, None);

    let unused_variables = cx.meta_word(DUMMY_SP,
        InternedString::new("unused_variables"));
    let allow = cx.meta_list(
        DUMMY_SP,
        InternedString::new("allow"), vec!(unused_variables));
    let allow_noncamel = cx.attribute(DUMMY_SP, allow);

    self.item_fn(cx, DUMMY_SP, "platformtree_main", &[allow_noncamel], body)
  }

  fn emit_start(&self, cx: &ExtCtxt) -> P<ast::Item> {
      /*
      let argc = ast::Arg {
          ty: quote_ty!(cx, isize),
          pat: cx.pat_wild(DUMMY_SP),
          id: ast::DUMMY_NODE_ID,
      };
      let argv = ast::Arg {
          ty: quote_ty!(cx, *const *const u8),
          pat: cx.pat_wild(DUMMY_SP),
          id: ast::DUMMY_NODE_ID,
      };
      cx.item_fn(
          DUMMY_SP,
          cx.ident_of("start"),
          vec!(argc, argv),
          quote_ty!(cx, isize),
          body)
      */
      quote_item!(cx,
          #[start]
          fn start(_: isize, _: *const *const u8) -> isize {
              unsafe {
                  platformtree_main();
              }
              0
          }
      ).unwrap()
  }

  fn emit_morestack(&self, cx: &ExtCtxt) -> P<ast::Item> {
    let stmt = cx.stmt_expr(quote_expr!(&*cx,
        core::intrinsics::abort()
        // or
        // zinc::os::task::morestack();
    ));
    let empty_span = DUMMY_SP;
    let body = cx.block(empty_span, vec!(stmt), None);
    self.item_fn(cx, empty_span, "__morestack", &[], body)
  }

  pub fn emit_items(&self, cx: &ExtCtxt) -> Vec<P<ast::Item>> {
    let non_camel_case_types = cx.meta_word(DUMMY_SP,
        InternedString::new("non_camel_case_types"));
    let allow = cx.meta_list(
        DUMMY_SP,
        InternedString::new("allow"), vec!(non_camel_case_types));
    let allow_noncamel = cx.attribute(DUMMY_SP, allow);
    let pt_mod_item = cx.item_mod(DUMMY_SP, DUMMY_SP, cx.ident_of("pt"),
        vec!(allow_noncamel), self.type_items.clone());

    if self.type_items.len() > 1 {
      vec!(pt_mod_item, self.emit_main(cx), self.emit_start(cx), self.emit_morestack(cx))
    } else {
      vec!(self.emit_main(cx), self.emit_start(cx), self.emit_morestack(cx))
    }
  }

  fn item_fn(&self, cx: &ExtCtxt, span: Span, name: &str,
      local_attrs: &[ast::Attribute], body: P<ast::Block>)
      -> P<ast::Item> {
    let attr_no_mangle = cx.attribute(span, cx.meta_word(
        span, InternedString::new("no_mangle")));
    let mut attrs = vec!(attr_no_mangle);
    attrs.push_all(local_attrs);

    P(ast::Item {
      ident: cx.ident_of(name),
      attrs: attrs,
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemFn(
          cx.fn_decl(Vec::new(), cx.ty(DUMMY_SP, ast::Ty_::TyTup(Vec::new()))),
          ast::Unsafety::Unsafe,
          abi::Rust, // TODO(farcaller): should this be abi::C?
          empty_generics(),
          body),
      vis: ast::Public,
      span: span,
    })
  }
}

pub struct TokenString(pub String);

impl ToTokens for TokenString {
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
    let &TokenString(ref s) = self;
    (cx as &ExtParseUtils).parse_tts(s.clone())
  }
}

pub fn add_node_dependency(node: &Rc<node::Node>, dep: &Rc<node::Node>) {
  let mut depends_on = node.depends_on.borrow_mut();
  depends_on.deref_mut().push(dep.downgrade());

  let mut rev_depends_on = dep.rev_depends_on.borrow_mut();
  rev_depends_on.push(node.downgrade());
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
