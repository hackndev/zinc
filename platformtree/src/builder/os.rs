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

use std::collections::HashSet;
use std::rc::Rc;
use syntax::ast;
use syntax::codemap::{respan, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token::intern;
use syntax::ptr::P;

use builder::meta_args::{ToTyHash, set_ty_params_for_task};
use node;
use super::{Builder, TokenString, add_node_dependency};

pub fn attach(builder: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(verify as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
  let mcu_node = builder.pt.get_by_path("mcu").unwrap();

  let maybe_task_node = node.get_by_path("single_task");
  if maybe_task_node.is_some() {
    let task_node = maybe_task_node.unwrap();
    task_node.materializer.set(Some(build_single_task as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
    add_node_dependency(&node, &task_node);
    add_node_dependency(&task_node, &mcu_node);

    let maybe_args_node = task_node.get_by_path("args");
    if maybe_args_node.is_some() {
      let args_node = maybe_args_node.unwrap();
      for (_, ref attr) in args_node.attributes.borrow().iter() {
        match attr.value {
          node::RefValue(ref refname) => {
            let refnode = builder.pt.get_by_name(refname.as_slice()).unwrap();
            add_node_dependency(&task_node, &refnode);
          },
          _ => (),
        }
      }
    }
  }
}

pub fn verify(_: &mut Builder, cx: &mut ExtCtxt, node: Rc<node::Node>) {
  node.expect_no_attributes(cx);
  node.expect_subnodes(cx, &["single_task"]);
  if node.get_by_path("single_task").is_none() {
    cx.parse_sess().span_diagnostic.span_err(node.name_span,
        "subnode `single_task` must be present");
  }
}

fn build_single_task(builder: &mut Builder, cx: &mut ExtCtxt,
    node: Rc<node::Node>) {
  let some_loop_fn = node.get_required_string_attr(cx, "loop");
  match some_loop_fn {
    Some(loop_fn) => {
      let args_node = node.get_by_path("args");
      let args = match args_node.and_then(|args| {
        Some(build_args(builder, cx, &loop_fn, args))
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
    struct_name: &String, node: Rc<node::Node>) -> P<ast::Expr> {
  let mut fields = vec!();
  let mut expr_fields = vec!();
  let node_attr = node.attributes.borrow();
  let mut ty_params = HashSet::new();

  // this is a bit slower than for (k, v) in node.attributes.iter(), but we need
  // to preserve sort order to make reasonably simple test code
  let mut all_keys = Vec::new();
  for k in node_attr.keys() { all_keys.push(k.clone()) };
  all_keys.sort();

  for k in all_keys.iter() {
    let v = &(*node_attr)[*k];

    let (ty, val) = match v.value {
      node::IntValue(i) =>
        (cx.ty_ident(DUMMY_SP, cx.ident_of("u32")),
            quote_expr!(&*cx, $i)),
      node::BoolValue(b) =>
        (cx.ty_ident(DUMMY_SP, cx.ident_of("bool")),
            quote_expr!(&*cx, $b)),
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
        let reftype = refnode.type_name().unwrap();
        let refparams = refnode.type_params();
        for param in refparams.iter() {
          if !param.as_slice().starts_with("'") {
            ty_params.insert(param.clone());
          }
        }
        let val_slice = TokenString(rname.clone());
        let a_lifetime = cx.lifetime(DUMMY_SP, intern("'a"));
        (cx.ty_rptr(
          DUMMY_SP,
          cx.ty_path(type_name_as_path(cx, reftype.as_slice(), refparams)),
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
  let mut collected_params = vec!();
  let mut ty_params_vec = vec!();
  for ty in ty_params.iter() {
    let typaram = cx.typaram(
        DUMMY_SP,
        cx.ident_of(ty.to_tyhash().as_slice()),
        OwnedSlice::from_vec(vec!(
          ast::RegionTyParamBound(cx.lifetime(DUMMY_SP, intern("'a")))
        )),
        None);
    collected_params.push(typaram);
    ty_params_vec.push(ty.clone());
  }

  set_ty_params_for_task(cx, struct_name.as_slice(), ty_params_vec);
  let struct_item = P(ast::Item {
    ident: name_ident,
    attrs: vec!(),
    id: ast::DUMMY_NODE_ID,
    node: ast::ItemStruct(P(ast::StructDef {
      fields: fields,
      ctor_id: None,
    }), ast::Generics {
      lifetimes: vec!(cx.lifetime_def(DUMMY_SP, intern("'a"), vec!())),
      ty_params: OwnedSlice::from_vec(collected_params),
      where_clause: ast::WhereClause {
        id: ast::DUMMY_NODE_ID,
        predicates: vec!(),
      }
    }),
    vis: ast::Public,
    span: DUMMY_SP,
  });
  builder.add_type_item(struct_item);

  cx.expr_addr_of(DUMMY_SP,
      cx.expr_struct(
          DUMMY_SP,
          cx.path(DUMMY_SP, vec!(cx.ident_of("pt"), name_ident)),
          expr_fields))
}

fn type_name_as_path(cx: &ExtCtxt, ty: &str, params: Vec<String>) -> ast::Path {
  let mut lifetimes = vec!();
  let mut types = vec!();
  for p in params.iter() {
    let slice = p.as_slice();
    if slice.starts_with("'") {
      let lifetime = cx.lifetime(DUMMY_SP, intern(slice));
      lifetimes.push(lifetime);
    } else {
      let path = cx.ty_path(type_name_as_path(cx, p.to_tyhash().as_slice(), vec!()));
      types.push(path);
    }
  }
  cx.path_all(DUMMY_SP, false,
      ty.split("::").map(|t| cx.ident_of(t)).collect(),
      lifetimes,
      types,
      vec!())
}

#[cfg(test)]
mod test {
  use std::ops::Deref;
  use syntax::codemap::DUMMY_SP;
  use syntax::ext::build::AstBuilder;

  use builder::Builder;
  use super::build_single_task;
  use test_helpers::{assert_equal_source, with_parsed};

  #[test]
  fn builds_single_task_os_loop() {
    with_parsed("
      single_task {
        loop = \"run\";
      }", |cx, failed, pt| {
      let mut builder = Builder::new(pt.clone(), cx);
      build_single_task(&mut builder, cx, pt.get_by_path("single_task").unwrap().clone());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);

      assert_equal_source(builder.main_stmts[0].deref(),
          "loop {
            run();
          }");
    });
  }

  #[test]
  fn builds_single_task_with_args() {
    with_parsed("
      single_task {
        loop = \"run\";
        args {
          a = 1;
          b = \"a\";
          c = &named;
        }
      }

      named@ref;
      ", |cx, failed, pt| {
      let mut builder = Builder::new(pt.clone(), cx);
      pt.get_by_path("ref").unwrap().set_type_name("hello::world::Struct".to_string());

      build_single_task(&mut builder, cx, pt.get_by_path("single_task").unwrap().clone());
      assert!(unsafe{*failed} == false);
      assert!(builder.main_stmts.len() == 1);
      assert!(builder.type_items.len() == 2);

      // XXX: builder.type_items[0] is `use zinc;` now
      assert_equal_source(cx.stmt_item(DUMMY_SP, builder.type_items[1].clone()).deref(),
          "pub struct run_args<'a> {
            pub a: u32,
            pub b: &'static str,
            pub c: &'a hello::world::Struct,
          }");

      assert_equal_source(builder.main_stmts[0].deref(),
          "loop {
            run(&pt::run_args {
              a: 1usize,
              b: \"a\",
              c: &named,
            });
          }");
    });
  }
}
