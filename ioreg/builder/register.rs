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

use std::gc::GC;
use std::iter::FromIterator;
use syntax::ast;
use syntax::ast::P;
use syntax::ast_util::empty_generics;
use syntax::codemap::{respan, mk_sp};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::token;

use super::Builder;
use super::utils;
use super::super::node;

/// A visitor to build the struct for each register
pub struct BuildRegStructs<'a, 'b, 'c> {
  builder: &'a mut Builder,
  cx: &'b ExtCtxt<'c>,
}

impl<'a, 'b, 'c> node::RegVisitor for BuildRegStructs<'a, 'b, 'c> {
  fn visit_prim_reg(&mut self, path: &Vec<String>, reg: &node::Reg,
                    width: node::RegWidth, fields: &Vec<node::Field>) {
    for field in fields.iter() {
      match build_field_type(self.cx, path, reg, field) {
        Some(item) => self.builder.push_item(item),
        None       => {}
      }
    }

    let reg_struct = build_reg_struct(self.cx, path, reg, width);
    self.builder.push_item(reg_struct);
  }
}

impl<'a, 'b, 'c> BuildRegStructs<'a, 'b, 'c> {
  pub fn new(builder: &'a mut Builder, cx: &'b ExtCtxt<'c>)
             -> BuildRegStructs<'a, 'b, 'c> {
    BuildRegStructs {builder: builder, cx: cx}
  }
}

/// Build a field type if necessary (e.g. in the case of an `EnumField`)
fn build_field_type<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
                        reg: &node::Reg, field: &node::Field)
                        -> Option<P<ast::Item>> {
  match field.ty.node {
    node::EnumField { variants: ref variants, .. } => {
      // FIXME(bgamari): We construct a path, then only take the last
      // segment, this could be more efficient
      let name: ast::Ident =
        utils::field_type_path(cx, path, reg, field)
        .segments.last().unwrap().identifier;
      let enum_def: ast::EnumDef = ast::EnumDef {
        variants: FromIterator::from_iter(
          variants.iter().map(|v| box(GC) build_enum_variant(cx, v))),
      };
      let attrs: Vec<ast::Attribute> = vec!(
        utils::list_attribute(cx, "deriving", vec!("FromPrimitive")),
        utils::list_attribute(cx, "allow",
                              vec!("uppercase_variables",
                                   "dead_code",
                                   "non_camel_case_types",
                                   "missing_doc")));
      let item: P<ast::Item> = box(GC) ast::Item {
        ident: name,
        id: ast::DUMMY_NODE_ID,
        node: ast::ItemEnum(enum_def, empty_generics()),
        vis: ast::Public,
        attrs: attrs,
        span: field.ty.span,
      };
      Some(item)
    },
    _ => None,
  }
}

/// Produce a register struct if necessary (for primitive typed registers).
/// In this case `None` indicates no struct is necessary, not failure.
/// For instance,
///
///     pub struct REG {_value: u32}
fn build_reg_struct<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, _width: node::RegWidth) -> P<ast::Item> {
  let packed_ty = 
    utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive reg");

  let reg_doc = match reg.docstring {
    Some(d) => token::get_ident(d.node).get().into_string(),
    None => "no documentation".into_string(),
  };
  let docstring = format!("Register `{}`: {}",
                          reg.name.node,
                          reg_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let ty_name = utils::path_ident(cx, path);
  let item = quote_item!(cx,
    $doc_attr
    #[allow(non_camel_case_types)]
    pub struct $ty_name {
      value: VolatileCell<$packed_ty>,
    }
  );
  item.unwrap()
}

/// Build a variant of an `EnumField`
fn build_enum_variant<'a>(cx: &'a ExtCtxt,
                          variant: &node::Variant) -> ast::Variant {
  let doc = match variant.docstring {
    Some(d) => token::get_ident(d.node).get().into_string(),
    None => "no documentation".into_string(),
  };
  let docstring = format!("`0x{:x}`. {}",
                          variant.value.node,
                          doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));
  respan(
    mk_sp(variant.name.span.lo, variant.value.span.hi),
    ast::Variant_ {
      name: cx.ident_of(variant.name.node.as_slice()),
      attrs: vec!(doc_attr),
      kind: ast::TupleVariantKind(Vec::new()),
      id: ast::DUMMY_NODE_ID,
      disr_expr: Some(utils::expr_int(cx, variant.value.node as i64)),
      vis: ast::Inherited,
    }
  )
}
