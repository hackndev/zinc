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
use syntax::codemap::{Span, respan, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token::{InternedString, intern};

use lpc17xx_pt;
use node;

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

pub fn build_platformtree(cx: &mut ExtCtxt, pt: &Gc<node::PlatformTree>) -> Builder {
  let mut builder = Builder::new(pt);

  if !pt.expect_subnodes(cx, ["mcu", "os"]) {
    return builder;  // TODO(farcaller): report error?
  }

  match pt.get_by_path("mcu") {
    Some(node) => build_mcu(&mut builder, cx, node),
    None => (),  // TODO(farcaller): should it actaully fail?
  }

  match pt.get_by_path("os") {
    Some(node) => build_os(&mut builder, cx, node),
    None => {
      // TODO(farcaller): provide span for whole PT?
      cx.parse_sess().span_diagnostic.span_err(DUMMY_SP,
          "root node `os` must be present");
    }
  }

  builder
}

fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt, node: &Gc<node::Node>) {
  match node.name {
    Some(ref name) => {
      match name.as_slice() {
        "lpc17xx" => lpc17xx_pt::build_mcu(builder, cx, node),
        other => {
          cx.parse_sess().span_diagnostic.span_err(node.name_span,
              format!("unknown mcu `{}`", other).as_slice());
        },
      }
    },
    None => {
      cx.parse_sess().span_diagnostic.span_err(node.name_span,
          "`mcu` node must have a name");
    },
  }
}

pub fn build_os(builder: &mut Builder, cx: &mut ExtCtxt, node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) ||
     !node.expect_subnodes(cx, ["single_task"]) {
    return;
  }

  let some_single_task = node.get_by_path("single_task");
  match some_single_task {
    Some(single_task) => {
      build_single_task(builder, cx, single_task);
    },
    None => {
      cx.parse_sess().span_diagnostic.span_err(node.name_span,
          "subnode `single_task` must be present");
    }
  }
}

fn build_single_task(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  let some_loop_fn = node.get_required_string_attr(cx, "loop");
  match some_loop_fn {
    Some(loop_fn) => {
      let args_node = node.get_by_path("args");
      let args = match args_node.and_then(|args| {
        Some(build_args(builder, cx, loop_fn, args))
      }) {
        None => vec!(),
        Some(arg) => vec!(arg),
      };

      let call_expr = cx.expr_call_ident(
          node.get_attr("loop").value_span,
          cx.ident_of(loop_fn.as_slice()),
          args);
      let loop_stmt = quote_stmt!(&*cx, loop { $call_expr; } );
      builder.add_main_statement(loop_stmt);
    },
    None => (),
  }
}

fn build_args(builder: &mut Builder, cx: &mut ExtCtxt,
    struct_name: &String, node: &Gc<node::Node>) -> Gc<ast::Expr> {
  let mut fields = Vec::new();
  let mut expr_fields = Vec::new();

  // this is a bit slower than for (k, v) in node.attributes.iter(), but we need
  // to preserve sort order to make reasonably simple test code
  let mut all_keys = Vec::new();
  for k in node.attributes.keys() { all_keys.push(k.clone()) };
  all_keys.sort();

  for k in all_keys.iter() {
    let v = node.attributes.get(k);

    let (ty, val) = match v.value {
      node::IntValue(i) =>
        (cx.ty_ident(DUMMY_SP, cx.ident_of("u32")),
            quote_expr!(&*cx, $i)),
      node::StrValue(ref string)  => {
        let static_lifetime = cx.lifetime(DUMMY_SP, intern("'static"));
        let val_slice = string.as_slice();
        (cx.ty_rptr(
          DUMMY_SP,
          cx.ty_ident(DUMMY_SP, cx.ident_of("str")),
          Some(static_lifetime),
          ast::MutImmutable), quote_expr!(&*cx, $val_slice))
      },
      node::RefValue(ref rname)  => {
        let refnode = builder.pt.get_by_name(rname.as_slice()).unwrap();
        let reftype = refnode.type_name.get().unwrap();
        let val_slice = TokenString::new(rname.clone());
        let a_lifetime = cx.lifetime(DUMMY_SP, intern("'a"));
        (cx.ty_rptr(
          DUMMY_SP,
          cx.ty_path(type_name_as_path(cx, reftype), None),
          Some(a_lifetime),
          ast::MutImmutable), quote_expr!(&*cx, &$val_slice))
      },
    };
    let name_ident = cx.ident_of(k.as_slice());
    let sf = ast::StructField_ {
      kind: ast::NamedField(name_ident, ast::Public),
      id: ast::DUMMY_NODE_ID,
      ty: ty,
      attrs: vec!(),
    };

    fields.push(respan(DUMMY_SP, sf));
    expr_fields.push(cx.field_imm(DUMMY_SP, name_ident, val));
  }

  let name_ident = cx.ident_of(format!("{}_args", struct_name).as_slice());
  let a_lifetime = cx.lifetime(DUMMY_SP, intern("'a"));
  let struct_item = box(GC) ast::Item {
    ident: name_ident,
    attrs: vec!(),
    id: ast::DUMMY_NODE_ID,
    node: ast::ItemStruct(box(GC) ast::StructDef {
      fields: fields,
      ctor_id: None,
      super_struct: None,
      is_virtual: false,
    }, ast::Generics {
      lifetimes: vec!(a_lifetime),
      ty_params: OwnedSlice::from_vec(vec!()),
    }),
    vis: ast::Public,
    span: DUMMY_SP,
  };
  builder.add_type_item(struct_item);

  cx.expr_addr_of(DUMMY_SP,
      cx.expr_struct(
          DUMMY_SP,
          cx.path(DUMMY_SP, vec!(cx.ident_of("pt"), name_ident)),
          expr_fields))
}

fn type_name_as_path(cx: &ExtCtxt, ty: &str) -> ast::Path {
  cx.path(DUMMY_SP, ty.split_str("::").map(|t| cx.ident_of(t)).collect())
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
