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
use syntax::ast;
use syntax::codemap::{respan, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token::intern;

use node;

use super::{Builder, TokenString};

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

#[cfg(test)]
mod test {
  use syntax::codemap::DUMMY_SP;
  use syntax::ext::build::AstBuilder;

  use builder::Builder;
  use super::build_os;
  use test_helpers::{assert_equal_source, with_parsed};

  #[test]
  fn builds_single_task_os_loop() {
    with_parsed("os {
        single_task {
          loop = \"run\";
        }
      }", |cx, failed, pt| {
      let mut builder = Builder::new(pt);
      build_os(&mut builder, cx, pt.get_by_path("os").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);

      assert_equal_source(builder.main_stmts.get(0),
          "loop {
            run();
          }");
    });
  }

  #[test]
  fn builds_single_task_with_args() {
    with_parsed("os {
        single_task {
          loop = \"run\";
          args {
            a = 1;
            b = \"a\";
            c = &named;
          }
        }
      }

      named@ref;
      ", |cx, failed, pt| {
      let mut builder = Builder::new(pt);
      pt.get_by_path("ref").unwrap().type_name.set(Some("hello::world::Struct"));

      build_os(&mut builder, cx, pt.get_by_path("os").unwrap());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);
      assert!(builder.type_items.len() == 1);

      assert_equal_source(&cx.stmt_item(DUMMY_SP, *builder.type_items.get(0)),
          "pub struct run_args<'a> {
            pub a: u32,
            pub b: &'static str,
            pub c: &'a hello::world::Struct,
          }");

      assert_equal_source(builder.main_stmts.get(0),
          "loop {
            run(&pt::run_args {
              a: 1u,
              b: \"a\",
              c: &named,
            });
          }");
    });
  }
}
