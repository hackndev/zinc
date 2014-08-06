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

use syntax::ast;
use syntax::ast::P;
use syntax::codemap::DUMMY_SP;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::token;

use super::Builder;
use super::utils;
use super::super::node;

/// A visitor to build accessor functions for each register struct
pub struct BuildAccessors<'a, 'b, 'c> {
  builder: &'a mut Builder,
  cx: &'b ExtCtxt<'c>,
}

impl<'a, 'b, 'c> node::RegVisitor for BuildAccessors<'a, 'b, 'c> {
  fn visit_prim_reg(&mut self, path: &Vec<String>, reg: &node::Reg,
                    _width: node::RegWidth, fields: &Vec<node::Field>) {
    let item = build_get_fn(self.cx, path, reg);
    self.builder.push_item(item);

    for field in fields.iter() {
      match build_field_accessors(self.cx, path, reg, field) {
        Some(item) => self.builder.push_item(item),
        None       => {}
      }
    }
  }
}

impl<'a, 'b, 'c> BuildAccessors<'a, 'b, 'c> {
  pub fn new(builder: &'a mut Builder, cx: &'b ExtCtxt<'c>)
             -> BuildAccessors<'a, 'b, 'c> {
    BuildAccessors {builder: builder, cx: cx}
  }
}

fn build_field_accessors<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
                             reg: &node::Reg, field: &node::Field)
                             -> Option<P<ast::Item>>
{
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(DUMMY_SP, utils::path_ident(cx, path));

  let items = match field.access {
    node::ReadWrite => vec!(build_field_set_fn(cx, path, reg, field),
                            build_field_get_fn(cx, path, reg, field)),
    node::ReadOnly  => vec!(build_field_get_fn(cx, path, reg, field)),
    node::WriteOnly => vec!(build_field_set_fn(cx, path, reg, field)),
    node::SetToClear => vec!(build_field_clear_fn(cx, path, reg, field)),
  };

  let access_tag = match field.access {
    node::ReadWrite => "read/write",
    node::ReadOnly  => "read-only",
    node::WriteOnly => "write-only",
    node::SetToClear => "set-to-clear",
  };

  let field_doc = match field.docstring {
    Some(ref d) => {
      let s = token::get_ident(d.node);
      s.get().into_string()
    },
    None => "no documentation".into_string()
  };
  let docstring = format!("*[{}]* Field `{}`: {}",
                          access_tag,
                          field.name.node,
                          field_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  quote_item!(cx,
    $doc_attr
    impl $reg_ty {
      $items
    }
  )
}

fn build_get_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>, _reg: &node::Reg)
                    -> P<ast::Item>
{
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(DUMMY_SP, utils::path_ident(cx, path));
  let getter_ty = utils::getter_name(cx, path);
  let item = quote_item!(cx,
    impl $reg_ty {
      #[allow(dead_code)]
      pub fn get(&'static self) -> $getter_ty {
        $getter_ty::new(self)
      }
    }
    );
  item.unwrap()
}

fn build_field_set_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
                          reg: &node::Reg, field: &node::Field)
                          -> P<ast::Method>
{
  let fn_name =
    cx.ident_of((String::from_str("set_")+field.name.node).as_slice());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field), None);
  let setter_ty = utils::setter_name(cx, path);
  if field.count.node == 1 {
    quote_method!(cx,
      #[allow(dead_code, missing_doc)]
      pub fn $fn_name(&'static self, new_value: $field_ty) -> $setter_ty {
        let mut setter: $setter_ty = $setter_ty::new(self);
        setter.$fn_name(new_value);
        setter
      }
    )
  } else {
    quote_method!(cx,
      #[allow(dead_code, missing_doc)]
      pub fn $fn_name(&'static self, idx: uint, new_value: $field_ty) -> $setter_ty {
        let mut setter: $setter_ty = $setter_ty::new(self);
        setter.$fn_name(idx, new_value);
        setter
      }
    )
  }
}

fn build_field_get_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
                          reg: &node::Reg, field: &node::Field)
                          -> P<ast::Method>
{
  let fn_name = cx.ident_of(field.name.node.as_slice());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field), None);
  let getter_ty = utils::getter_name(cx, path);
  if field.count.node == 1 {
    quote_method!(cx,
      #[allow(dead_code, missing_doc)]
      pub fn $fn_name(&'static self) -> $field_ty {
        $getter_ty::new(self).$fn_name()
      }
    )
  } else {
    quote_method!(cx,
      #[allow(dead_code, missing_doc)]
      pub fn $fn_name(&'static self, idx: uint) -> $field_ty {
        $getter_ty::new(self).$fn_name(idx)
      }
    )
  }
}

fn build_field_clear_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
                            _reg: &node::Reg, field: &node::Field)
                            -> P<ast::Method>
{
  let fn_name =
    cx.ident_of((String::from_str("clear_")+field.name.node).as_slice());
  let setter_ty = utils::setter_name(cx, path);
  if field.count.node == 1 {
    quote_method!(cx,
      #[allow(dead_code, missing_doc)]
      pub fn $fn_name(&'static self) -> $setter_ty {
        let mut setter: $setter_ty = $setter_ty::new(self);
        setter.$fn_name();
        setter
      }
    )
  } else {
    quote_method!(cx,
      #[allow(dead_code, missing_doc)]
      pub fn $fn_name(&'static self, idx: uint) -> $setter_ty {
        let mut setter: $setter_ty = $setter_ty::new(self);
        setter.$fn_name(idx);
        setter
      }
    )
  }
}
