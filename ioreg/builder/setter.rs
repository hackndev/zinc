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
use syntax::ext::base::ExtCtxt;
use syntax::codemap::DUMMY_SP;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::token;

use super::Builder;
use super::super::node;
use super::utils;

/// A visitor to build the field setters for primitive registers
pub struct BuildSetters<'a, 'b, 'c> {
  builder: &'a mut Builder,
  cx: &'b ExtCtxt<'c>,
}

impl<'a, 'b, 'c> BuildSetters<'a, 'b, 'c> {
  pub fn new(builder: &'a mut Builder, cx: &'b ExtCtxt<'c>)
             -> BuildSetters<'a, 'b, 'c> {
    BuildSetters { builder: builder, cx: cx }
  }
}

impl<'a, 'b, 'c> node::RegVisitor for BuildSetters<'a, 'b, 'c> {
  fn visit_prim_reg<'a>(&'a mut self, path: &Vec<String>,
      reg: &'a node::Reg, _width: node::RegWidth, fields: &Vec<node::Field>)
  {
    if fields.iter().any(|f| f.access != node::ReadOnly) {
      let it = build_type(self.cx, path, reg, fields);
      self.builder.push_item(it);

      let it = build_drop(self.cx, path, reg, fields);
      self.builder.push_item(it);

      let it = build_impl(self.cx, path, reg, fields);
      self.builder.push_item(it);
    }
  }
}

fn build_type<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, _fields: &Vec<node::Field>) -> P<ast::Item>
{
  let packed_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let name = utils::setter_name(cx, path);
  let reg_ty = cx.ty_ident(DUMMY_SP, utils::path_ident(cx, path));

  let reg_doc = match reg.docstring {
    Some(d) => d.node,
    None => cx.ident_of("no documentation"),
  };
  let docstring = format!("Update value of `{}` register: {}",
                          reg.name.node,
                          reg_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let item = quote_item!(cx,
    $doc_attr
    #[allow(non_camel_case_types)]
    pub struct $name {
      value: $packed_ty,
      mask: $packed_ty,
      reg: &'static $reg_ty,
    }
  );
  item.unwrap()
}

fn build_new<'a>(cx: &'a ExtCtxt, path: &Vec<String>)
                 -> P<ast::Item> {
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(DUMMY_SP, utils::path_ident(cx, path));
  let setter_ty: P<ast::Ty> = cx.ty_ident(DUMMY_SP,
                                          utils::setter_name(cx, path));
  let item = quote_item!(cx,
    #[doc="Create a new updater"]
    pub fn new(reg: &'static $reg_ty) -> $setter_ty {
      $setter_ty {
        value: 0,
        mask: 0,
        reg: reg,
      }
    });
  item.unwrap()
}

fn build_drop<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, fields: &Vec<node::Field>) -> P<ast::Item>
{
  let setter_ty: P<ast::Ty> = cx.ty_ident(DUMMY_SP,
                                          utils::setter_name(cx, path));
  let unpacked_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");

  // ensure we don't unintentionally clear a set-to-clear flag
  let mut clear: u32 = 0;
  for f in fields.iter() {
    match f.access {
      node::SetToClear => {
        let mask = 1 << (f.count.node * f.width) - 1;
        clear |= mask;
      },
      _ => {},
    }
  }

  let item = quote_item!(cx,
    #[unsafe_destructor]
    #[doc = "This performs the register update"]
    impl Drop for $setter_ty {
      fn drop(&mut self) {
        let clear_mask: $unpacked_ty = $clear as $unpacked_ty;
        if self.mask != 0 {
          let v: $unpacked_ty = self.reg.value.get() & ! clear_mask & ! self.mask;
          self.reg.value.set(self.value | v);
        }
      }
    }
  );
  item.unwrap()
}

fn build_done<'a>(cx: &'a ExtCtxt) -> P<ast::Method>
{
  quote_method!(cx,
    #[doc="Commit changes to register. This is to allow chains of `set_*` \
           invocations to be used as a statement."]
    pub fn done(self) {}
  )
}

fn build_impl<'a>(cx: &'a ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                  fields: &Vec<node::Field>) -> P<ast::Item>
{
  let new = build_new(cx, path);
  let setter_ty: P<ast::Ty> = cx.ty_ident(
    DUMMY_SP,
    utils::setter_name(cx, path));
  let methods: Vec<P<ast::Method>> =
    FromIterator::from_iter(
      fields.iter()
        .filter_map(|field| build_field_fn(cx, path, reg, field)));
  let done: P<ast::Method> = build_done(cx);
  let impl_ = quote_item!(cx,
    #[allow(dead_code)]
    impl $setter_ty {
      $new
      $methods
      $done
    }
    );
  impl_.unwrap()
}

fn build_field_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                      field: &node::Field) -> Option<P<ast::Method>>
{
  match field.access {
    node::ReadOnly => None,
    node::SetToClear => Some(build_field_clear_fn(cx, path, reg, field)),
    _ => Some(build_field_set_fn(cx, path, reg, field)),
  }
}

/// Build a setter for a field
fn build_field_set_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                          field: &node::Field) -> P<ast::Method>
{
  let setter_ty = utils::setter_name(cx, path);
  let unpacked_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let fn_name =
    cx.ident_of((String::from_str("set_")+field.name.node).as_slice());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field), None);
  let mask = utils::mask(cx, field);

  let field_doc = match field.docstring {
    Some(d) => token::get_ident(d.node).get().into_string(),
    None => "no documentation".into_string(),
  };
  let docstring = format!("Set value of `{}` field: {}",
                          field.name.node,
                          field_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  if field.count.node == 1 {
    let shift = utils::shift(cx, None, field);
    quote_method!(cx,
      $doc_attr
      pub fn $fn_name<'a>(&'a mut self, new_value: $field_ty)
          -> &'a mut $setter_ty {
        self.value |= (self.value & ! $mask) | ((new_value as $unpacked_ty) & $mask) << $shift;
        self.mask |= $mask << $shift;
        self
      }
    )
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    quote_method!(cx,
      $doc_attr
      pub fn $fn_name<'a>(&'a mut self, idx: uint, new_value: $field_ty)
        -> &'a mut $setter_ty {
          self.value |= (self.value & ! $mask) | ((new_value as $unpacked_ty) & $mask) << $shift;
          self.mask |= $mask << $shift;
          self
      }
    )
  }
}

fn build_field_clear_fn<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
    _: &node::Reg, field: &node::Field) -> P<ast::Method>
{
  let setter_ty = utils::setter_name(cx, path);
  let fn_name =
    cx.ident_of((String::from_str("clear_")+field.name.node).as_slice());
  let mask = utils::mask(cx, field);
  let docstring = match field.docstring {
    Some(d) => utils::doc_attribute(cx, token::get_ident(d.node)).node.to_tokens(cx),
    None => Vec::new(),
  };

  if field.count.node == 1 {
    let shift = utils::shift(cx, None, field);
    quote_method!(cx,
      $docstring
      pub fn $fn_name<'a>(&'a mut self) -> &'a mut $setter_ty {
          self.value |= $mask << $shift;
          self.mask |= $mask << $shift;
          self
      }
    )
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    quote_method!(cx,
      $docstring
      pub fn $fn_name<'a>(&'a mut self, idx: uint)
                          -> &'a mut $setter_ty {
          self.value |= $mask << $shift;
          self.mask |= $mask << $shift;
          self
      }
    )
  }
}
