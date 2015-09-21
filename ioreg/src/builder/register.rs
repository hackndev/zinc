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
use syntax::ast_util::empty_generics;
use syntax::codemap::{respan, mk_sp};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;

use super::Builder;
use super::utils;
use super::super::node;

/// A visitor to build the struct for each register
pub struct BuildRegStructs<'a> {
  builder: &'a mut Builder,
  cx: &'a ExtCtxt<'a>,
}

impl<'a> node::RegVisitor for BuildRegStructs<'a> {
  fn visit_prim_reg(&mut self, path: &Vec<String>, reg: &node::Reg,
                    fields: &Vec<node::Field>) {
    let width = match reg.ty {
      node::RegType::RegPrim(ref width, _) => width.node,
      _ => panic!("visit_prim_reg called with non-primitive register"),
    };
    for field in fields.iter() {
      for item in build_field_type(self.cx, path, reg, field).into_iter() {
        self.builder.push_item(item);
      }
    }

    for item in build_reg_struct(self.cx, path, reg, &width).into_iter() {
      self.builder.push_item(item);
    }
  }
}

impl<'a> BuildRegStructs<'a> {
  pub fn new(builder: &'a mut Builder, cx: &'a ExtCtxt<'a>)
             -> BuildRegStructs<'a> {
    BuildRegStructs {builder: builder, cx: cx}
  }
}

/// Build a field type if necessary (e.g. in the case of an `EnumField`)
fn build_field_type(cx: &ExtCtxt, path: &Vec<String>,
                    reg: &node::Reg, field: &node::Field)
                    -> Vec<P<ast::Item>> {
  match field.ty.node {
    node::FieldType::EnumField { ref variants, .. } => {
      // FIXME(bgamari): We construct a path, then only take the last
      // segment, this could be more efficient
      let name: ast::Ident =
        utils::field_type_path(cx, path, reg, field)
        .segments.last().unwrap().identifier;
      let enum_def: ast::EnumDef = ast::EnumDef {
        variants: FromIterator::from_iter(
          variants.iter().map(|v| P(build_enum_variant(cx, v)))),
      };
      let mut attrs: Vec<ast::Attribute> = vec!(
        utils::list_attribute(cx, "derive",
                              vec!("PartialEq"),
                              field.name.span),
        utils::list_attribute(cx, "allow",
                              vec!("dead_code",
                                   "non_camel_case_types",
                                   "missing_docs"),
                              field.name.span));
      
      // setting the enum's repr to the width of the register
      // (unless there's only 1 variant; in which case we omit 
      // the repr attribute due to E0083
      if variants.len() > 1 {
        let enum_repr = utils::reg_primitive_type_name(reg)
          .expect("Unexpected non-primitive reg");
        
        attrs.push(utils::list_attribute(cx, "repr",
                                         vec!(enum_repr),
                                         field.name.span));
      }
      
      let ty_item: P<ast::Item> = P(ast::Item {
        ident: name,
        id: ast::DUMMY_NODE_ID,
        node: ast::ItemEnum(enum_def, empty_generics()),
        vis: ast::Public,
        attrs: attrs,
        span: field.ty.span,
      });
      vec!(ty_item)
    },
    _ => Vec::new()
  }
}

/// Produce a register struct if necessary (for primitive typed registers).
/// In this case `None` indicates no struct is necessary, not failure.
/// For instance,
///
///     pub struct REG {_value: u32}
fn build_reg_struct(cx: &ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, _width: &node::RegWidth) -> Vec<P<ast::Item>> {
  let packed_ty =
    utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive reg");

  let reg_doc = match reg.docstring {
    Some(d) => d.node.name.to_string(),
    None => "no documentation".to_string(),
  };
  let docstring = format!("Register `{}`: {}",
                          reg.name.node,
                          reg_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let ty_name = utils::path_ident(cx, path);
  let item = quote_item!(cx,
    $doc_attr
    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    #[repr(C)]
    pub struct $ty_name {
      value: VolatileCell<$packed_ty>,
    }
  );
  let mut item: ast::Item = item.unwrap().deref().clone();
  item.span = reg.name.span;
  let copy_impl = quote_item!(cx, impl ::core::marker::Copy for $ty_name {}).unwrap();
  vec!(P(item), copy_impl)
}

/// Build a variant of an `EnumField`
fn build_enum_variant(cx: &ExtCtxt, variant: &node::Variant)
                      -> ast::Variant {
  let doc = match variant.docstring {
    Some(d) => d.node.name.to_string(),
    None => "no documentation".to_string(),
  };
  let docstring = format!("`0x{:x}`. {}",
                          variant.value.node,
                          doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));
  respan(
    mk_sp(variant.name.span.lo, variant.value.span.hi),
    ast::Variant_ {
      name: cx.ident_of(variant.name.node.as_str()),
      attrs: vec!(doc_attr),
      kind: ast::TupleVariantKind(Vec::new()),
      id: ast::DUMMY_NODE_ID,
      disr_expr: Some(utils::expr_int(cx, respan(variant.value.span,
                                                 variant.value.node as i64))),
    }
  )
}
