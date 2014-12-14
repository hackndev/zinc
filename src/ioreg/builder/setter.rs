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
use syntax::ptr::P;
use syntax::ext::base::ExtCtxt;
use syntax::codemap::DUMMY_SP;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::token;

use super::Builder;
use super::super::node;
use super::utils;

/// A visitor to build the field setters for primitive registers
pub struct BuildSetters<'a> {
  builder: &'a mut Builder,
  cx: &'a ExtCtxt<'a>,
}

impl<'a> BuildSetters<'a> {
  pub fn new(builder: &'a mut Builder, cx: &'a ExtCtxt<'a>)
      -> BuildSetters<'a> {
    BuildSetters { builder: builder, cx: cx }
  }
}

impl<'a> node::RegVisitor for BuildSetters<'a> {
  fn visit_prim_reg<'a>(&'a mut self, path: &Vec<String>,
      reg: &'a node::Reg, _width: &node::RegWidth, fields: &Vec<node::Field>)
  {
    if fields.iter().any(|f| f.access != node::Access::ReadOnly) {
      let it = build_type(self.cx, path, reg, fields);
      self.builder.push_item(it);

      let it = build_drop(self.cx, path, reg, fields);
      self.builder.push_item(it);

      let it = build_impl(self.cx, path, reg, fields);
      self.builder.push_item(it);
    }
  }
}

fn build_type(cx: &ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, _fields: &Vec<node::Field>) -> P<ast::Item>
{
  let packed_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let name = utils::setter_name(cx, path);
  let reg_ty = cx.ty_ident(DUMMY_SP, utils::path_ident(cx, path));

  let reg_doc = match reg.docstring {
    Some(d) => token::get_ident(d.node).get().into_string(),
    None => "no documentation".into_string(),
  };
  let docstring = format!("Update value of `{}` register: {}",
                          reg.name.node,
                          reg_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let item = quote_item!(cx,
    $doc_attr
    #[allow(non_camel_case_types)]
    pub struct $name<'a> {
      value: $packed_ty,
      mask: $packed_ty,
      reg: &'a $reg_ty,
    }
  );
  let mut item: ast::Item = item.unwrap().deref().clone();
  item.span = reg.name.span;
  P(item)
}

fn build_new<'a>(cx: &'a ExtCtxt, path: &Vec<String>)
                 -> P<ast::Item> {
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(DUMMY_SP, utils::path_ident(cx, path));
  let setter_ty: P<ast::Ty> = cx.ty_ident(DUMMY_SP,
                                          utils::setter_name(cx, path));
  let item = quote_item!(cx,
    #[doc="Create a new updater"]
    pub fn new(reg: &'a $reg_ty) -> $setter_ty {
      $setter_ty {
        value: 0,
        mask: 0,
        reg: reg,
      }
    });
  item.unwrap()
}

fn build_drop(cx: &ExtCtxt, path: &Vec<String>,
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
      node::Access::SetToClear => {
        let mask = 1 << (f.count.node * f.width) - 1;
        clear |= mask;
      },
      _ => {},
    }
  }

  // no need to read write-only registers
  let wo_reg: bool = fields.iter().all(|f| f.access == node::Access::WriteOnly);
  let initial_value =
    if wo_reg {
      quote_expr!(cx, 0)
    } else {
      quote_expr!(cx, self.reg.value.get())
    };

  let item = quote_item!(cx,
    #[unsafe_destructor]
    #[doc = "This performs the register update"]
    impl<'a> Drop for $setter_ty<'a> {
      fn drop(&mut self) {
        let clear_mask: $unpacked_ty = $clear as $unpacked_ty;
        if self.mask != 0 {
          let v: $unpacked_ty = $initial_value & ! clear_mask & ! self.mask;
          self.reg.value.set(self.value | v);
        }
      }
    }
  );
  item.unwrap()
}

fn build_done(cx: &ExtCtxt) -> P<ast::Method>
{
  quote_method!(cx,
    #[doc="Commit changes to register. This is to allow chains of `set_*` \
           invocations to be used as a statement."]
    pub fn done(self) {}
  )
}

fn build_impl(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
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
    impl<'a> $setter_ty<'a> {
      $new
      $methods
      $done
    }
    );
  impl_.unwrap()
}

fn build_field_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                  field: &node::Field) -> Option<P<ast::Method>>
{
  match field.access {
    node::Access::ReadOnly => None,
    node::Access::SetToClear => Some(build_field_clear_fn(cx, path, reg, field)),
    _ => Some(build_field_set_fn(cx, path, reg, field)),
  }
}

/// Build a setter for a field
fn build_field_set_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                      field: &node::Field) -> P<ast::Method>
{
  let setter_ty = utils::setter_name(cx, path);
  let unpacked_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let fn_name =
    cx.ident_of((String::from_str("set_")+field.name.node).as_slice());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field));
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
      pub fn $fn_name<'b>(&'b mut self, new_value: $field_ty)
          -> &'b mut $setter_ty<'a> {
        self.value |= (self.value & ! $mask) | ((new_value as $unpacked_ty) & $mask) << $shift;
        self.mask |= $mask << $shift;
        self
      }
    )
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    quote_method!(cx,
      $doc_attr
      pub fn $fn_name<'b>(&'b mut self, idx: uint, new_value: $field_ty)
          -> &'b mut $setter_ty<'a> {
        self.value |= (self.value & ! $mask) | ((new_value as $unpacked_ty) & $mask) << $shift;
        self.mask |= $mask << $shift;
        self
      }
    )
  }
}

fn build_field_clear_fn(cx: &ExtCtxt, path: &Vec<String>,
    _: &node::Reg, field: &node::Field) -> P<ast::Method>
{
  let setter_ty = utils::setter_name(cx, path);
  let fn_name =
    cx.ident_of((String::from_str("clear_")+field.name.node).as_slice());
  let mask = utils::mask(cx, field);

  let field_doc = match field.docstring {
    Some(d) => token::get_ident(d.node).get().into_string(),
    None => "no documentation".into_string(),
  };
  let docstring = format!("Clear `{}` flag: {}",
                          field.name.node,
                          field_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  if field.count.node == 1 {
    let shift = utils::shift(cx, None, field);
    quote_method!(cx,
      $doc_attr
      pub fn $fn_name<'b>(&'b mut self) -> &'b mut $setter_ty<'a> {
        self.value |= $mask << $shift;
        self.mask |= $mask << $shift;
        self
      }
    )
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    quote_method!(cx,
      $doc_attr
      pub fn $fn_name<'b>(&'b mut self, idx: uint) -> &'b mut $setter_ty<'a> {
        self.value |= $mask << $shift;
        self.mask |= $mask << $shift;
        self
      }
    )
  }
}
