// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

use std::iter::FromIterator;
use std::ops::Deref;

use syntax::ast;
use syntax::ptr::P;
use syntax::ext::base::ExtCtxt;
use syntax::codemap::{respan, Span};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::token;

use super::Builder;
use super::super::node;
use super::utils;

/// A visitor to build the field setters for primitive registers
pub struct BuildGetters<'a> {
  builder: &'a mut Builder,
  cx: &'a ExtCtxt<'a>,
}

impl<'a> BuildGetters<'a> {
  pub fn new(builder: &'a mut Builder, cx: &'a ExtCtxt<'a>)
      -> BuildGetters<'a> {
    BuildGetters { builder: builder, cx: cx }
  }
}

impl<'a> node::RegVisitor for BuildGetters<'a> {
  fn visit_prim_reg(&mut self, path: &Vec<String>,
                    reg: &node::Reg,
                    fields: &Vec<node::Field>) {
    if fields.iter().any(|f| f.access != node::Access::WriteOnly) {
      let it = build_type(self.cx, path, reg);
      self.builder.push_item(it);

      let it = build_impl(self.cx, path, reg, fields);
      self.builder.push_item(it);

      // Build Copy impl
      let ty_name = utils::getter_name(self.cx, path);
      let it = quote_item!(self.cx,
                           impl ::core::marker::Copy for $ty_name {});
      self.builder.push_item(it.unwrap());
    }
  }
}

fn reg_ty_span(reg: &node::Reg) -> Span {
  match reg.ty {
    node::RegType::RegPrim(ref width, _) => width.span,
    _ => reg.name.span,
  }
}

fn build_type(cx: &ExtCtxt, path: &Vec<String>,
              reg: &node::Reg) -> P<ast::Item>
{
  let packed_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let name = utils::getter_name(cx, path);
  let reg_doc = match reg.docstring {
    Some(d) => d.node.name.as_str(),
    None => "no documentation".to_string(),
  };
  let docstring = format!("`{}`: {}", reg.name.node, reg_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let item = quote_item!(cx,
    $doc_attr
    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    pub struct $name {
      value: $packed_ty,
    }
  );
  let mut item: ast::Item = item.unwrap().deref().clone();
  item.span = reg.name.span;
  P(item)
}

fn build_new(cx: &ExtCtxt, path: &Vec<String>,
             reg: &node::Reg) -> P<ast::ImplItem> {
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let getter_ident = utils::getter_name(cx, path);
  let getter_ty: P<ast::Ty> = cx.ty_ident(reg_ty_span(reg),
                                          getter_ident);
  let item = quote_item!(cx,
    impl $getter_ty {
      #[doc = "Create a getter reflecting the current value of the given register."]
      pub fn new(reg: & $reg_ty) -> $getter_ty {
        $getter_ident {
          value: reg.value.get(),
        }
      }
    }
  ).unwrap();
  utils::unwrap_impl_item(item)
}

/// Given an `Expr` of the given register's primitive type, return
/// an `Expr` of the field type
fn from_primitive(cx: &ExtCtxt, path: &Vec<String>, _: &node::Reg,
                  field: &node::Field, prim: P<ast::Expr>)
                  -> P<ast::Expr> {
  // Use bit_range_field for the span because it is to blame for the 
  // type of the register
  match field.ty.node {
    node::FieldType::UIntField => prim,
    node::FieldType::BoolField =>
      cx.expr_binary(field.bit_range_span, ast::BiNe,
                     prim, utils::expr_int(cx, respan(field.bit_range_span, 0))),
    node::FieldType::EnumField {opt_name: _, variants: ref vars} => {
      let mut arms: Vec<ast::Arm> = Vec::new();
      for v in vars.iter() {
        let mut name = path.clone();
        name.push(field.name.node.clone());
        let enum_ident = cx.ident_of(name.connect("_").as_str());
        let val_ident = cx.ident_of(v.name.node.as_str());
        let body = cx.expr_path(
          cx.path(v.name.span, vec!(enum_ident, val_ident)));
        let val: u64 = v.value.node;
        let lit = cx.expr_lit(
          v.value.span,
          ast::LitInt(val, ast::UnsuffixedIntLit(ast::Plus)));
        let arm = ast::Arm {
          attrs: vec!(),
          pats: vec!(
            P(ast::Pat {
              id: ast::DUMMY_NODE_ID,
              span: lit.span,
              node: ast::PatLit(lit),
            })
            ),
            guard: None,
            body: cx.expr_some(body.span, body),
        };
        arms.push(arm);
      }
      let wild_arm = ast::Arm {
        attrs: vec!(),
        pats: vec!(cx.pat_wild(field.name.span)),
        guard: None,
        body: cx.expr_none(field.name.span),
      };
      arms.push(wild_arm);
      let opt_expr = cx.expr_match(
        field.name.span,
        prim,
        arms);
      cx.expr_method_call(
        field.name.span,
        opt_expr,
        cx.ident_of("unwrap"),
        Vec::new())
    },
  }
}

fn build_impl(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
              fields: &Vec<node::Field>) -> P<ast::Item> {
  let getter_ty = utils::getter_name(cx, path);
  let new = build_new(cx, path, reg);
  let getters: Vec<P<ast::ImplItem>> =
    FromIterator::from_iter(
      fields.iter()
        .map(|field| build_field_get_fn(cx, path, reg, field)));

  let packed_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let get_raw: P<ast::ImplItem> = utils::unwrap_impl_item(quote_item!(cx,
    impl $getter_ty {
      #[doc = "Get the raw value of the register."]
      pub fn raw(&self) -> $packed_ty {
        self.value
      }
    }
  ).unwrap());

  let it = quote_item!(cx,
    #[allow(dead_code)]
    impl $getter_ty {
      $new
      $getters
      $get_raw
    }
  );
  let mut item: ast::Item = it.unwrap().deref().clone();
  item.span = reg.name.span;
  P(item)
}

/// Build a getter for a field
fn build_field_get_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                      field: &node::Field) -> P<ast::ImplItem>
{
  let fn_name = cx.ident_of(field.name.node.as_str());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field));
  let mask = utils::mask(cx, field);
  let field_doc = match field.docstring {
    Some(d) => d.node,
    None => cx.ident_of("no documentation"),
  };
  let docstring = format!("Get value of `{}` field: {}",
                          field.name.node,
                          field_doc.name.as_str());
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  if field.count.node == 1 {
    let shift = utils::shift(cx, None, field);
    let value = from_primitive(
      cx, path, reg, field,
      quote_expr!(cx, (self.value >> $shift) & $mask));
    utils::unwrap_impl_item(quote_item!(cx,
      impl X {
        $doc_attr
        pub fn $fn_name(&self) -> $field_ty {
          $value
        }
      }
    ).unwrap())
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    let value = from_primitive(
      cx, path, reg, field,
      quote_expr!(cx, (self.value >> $shift) & $mask));
    utils::unwrap_impl_item(quote_item!(cx,
      impl X {
        $doc_attr
        pub fn $fn_name(&self, idx: usize) -> $field_ty {
          $value
        }
      }
    ).unwrap())
  }
}
