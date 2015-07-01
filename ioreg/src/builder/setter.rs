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
  fn visit_prim_reg<'b>(&'b mut self, path: &Vec<String>,
      reg: &'b node::Reg, fields: &Vec<node::Field>)
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
  let reg_ty = cx.ty_ident(reg.name.span, utils::path_ident(cx, path));

  let reg_doc = match reg.docstring {
    Some(d) => token::get_ident(d.node).to_string(),
    None => "no documentation".to_string(),
  };
  let docstring = format!("Update value of `{}` register: {}",
                          reg.name.node,
                          reg_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let item = quote_item!(cx,
    $doc_attr
    #[allow(non_camel_case_types, dead_code)]
    pub struct $name<'a> {
      value: $packed_ty,
      mask: $packed_ty,
      write_only: bool,
      reg: &'a $reg_ty,
    }
  );
  let mut item: ast::Item = item.unwrap().deref().clone();
  item.span = reg.name.span;
  P(item)
}

fn build_new<'a>(cx: &'a ExtCtxt, path: &Vec<String>, reg: &node::Reg)
                 -> P<ast::ImplItem> {
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let setter_ident = utils::setter_name(cx, path);
  utils::unwrap_impl_item(quote_item!(cx,
    impl<'a> $setter_ident<'a> {
      #[doc="Create a new updater"]
      pub fn new(reg: &'a $reg_ty) -> $setter_ident<'a> {
        $setter_ident {
          value: 0,
          mask: 0,
          write_only: false,
          reg: reg,
        }
      }
    }
  ).unwrap())
}

fn build_new_ignoring_state<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
    reg: &node::Reg) -> P<ast::ImplItem> {
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let setter_ident = utils::setter_name(cx, path);
  utils::unwrap_impl_item(quote_item!(cx,
    impl<'a> $setter_ident<'a> {
      #[doc="Create a new updater that ignores current state"]
      pub fn new_ignoring_state(reg: &'a $reg_ty) -> $setter_ident<'a> {
        $setter_ident {
          value: 0,
          mask: 0,
          write_only: true,
          reg: reg,
        }
      }
    }
  ).unwrap())
}

fn build_drop(cx: &ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, fields: &Vec<node::Field>) -> P<ast::Item>
{
  let setter_ident = utils::setter_name(cx, path);
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
      quote_expr!(cx, if self.write_only { 0 } else { self.reg.value.get() })
    };

  let item = quote_item!(cx,
    #[doc = "This performs the register update"]
    impl<'a> Drop for $setter_ident<'a> {
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

fn build_done(ctx: &ExtCtxt, path: &Vec<String>) -> P<ast::ImplItem> {
  let setter_ident = utils::setter_name(ctx, path);
  utils::unwrap_impl_item(quote_item!(ctx,
    impl<'a> $setter_ident<'a> {
      #[doc = "Commit changes to register. Allows for chaining of set"]
      pub fn done(self) {}
    }
  ).unwrap())
}

fn build_impl(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
              fields: &Vec<node::Field>) -> P<ast::Item>
{
  let new = build_new(cx, path, reg);
  let new_is = build_new_ignoring_state(cx, path, reg);
  let setter_ident = utils::setter_name(cx, path);
  let methods: Vec<P<ast::ImplItem>> =
    FromIterator::from_iter(
      fields.iter()
        .filter_map(|field| build_field_fn(cx, path, reg, field)));
  let done = build_done(cx, path);
  quote_item!(cx,
    #[allow(dead_code)]
    impl<'a> $setter_ident<'a> {
      $new
      $new_is
      $methods
      $done
    }
  ).unwrap()
}

fn build_field_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                  field: &node::Field) -> Option<P<ast::ImplItem>>
{
  match field.access {
    node::Access::ReadOnly => None,
    node::Access::SetToClear => Some(build_field_clear_fn(cx, path, reg, field)),
    _ => Some(build_field_set_fn(cx, path, reg, field)),
  }
}

/// Build a setter for a field
fn build_field_set_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg,
                      field: &node::Field) -> P<ast::ImplItem>
{
  let setter_ty = utils::setter_name(cx, path);
  let unpacked_ty = utils::reg_primitive_type(cx, reg)
    .expect("Unexpected non-primitive register");
  let fn_name =
    cx.ident_of((String::from("set_")+field.name.node.as_str()).as_str());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field));
  let mask = utils::mask(cx, field);

  let field_doc = match field.docstring {
    Some(d) => token::get_ident(d.node).to_string(),
    None => "no documentation".to_string(),
  };
  let docstring = format!("Set value of `{}` field: {}",
                          field.name.node,
                          field_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  if field.count.node == 1 {
    let shift = utils::shift(cx, None, field);
    utils::unwrap_impl_item(quote_item!(cx,
      impl<'a> $setter_ty<'a> {
        $doc_attr
        pub fn $fn_name<'b>(&'b mut self,
                            new_value: $field_ty) -> &'b mut $setter_ty<'a> {
          self.value |= (self.value & ! $mask) | ((new_value as $unpacked_ty) & $mask) << $shift;
          self.mask |= $mask << $shift;
          self
        }
      }
    ).unwrap())
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    utils::unwrap_impl_item(quote_item!(cx,
      impl<'a> $setter_ty<'a> {
        $doc_attr
        pub fn $fn_name<'b>(&'b mut self,
                            idx: usize,
                            new_value: $field_ty) -> &'b mut $setter_ty<'a> {
          self.value |= (self.value & ! $mask) | ((new_value as $unpacked_ty) & $mask) << $shift;
          self.mask |= $mask << $shift;
          self
        }
      }
    ).unwrap())
  }
}

fn build_field_clear_fn(cx: &ExtCtxt, path: &Vec<String>,
    _: &node::Reg, field: &node::Field) -> P<ast::ImplItem>
{
  let setter_ty = utils::setter_name(cx, path);
  let fn_name =
    cx.ident_of((String::from("clear_")+field.name.node.as_str()).as_str());
  let mask = utils::mask(cx, field);

  let field_doc = match field.docstring {
    Some(d) => token::get_ident(d.node).to_string(),
    None => "no documentation".to_string(),
  };
  let docstring = format!("Clear `{}` flag: {}",
                          field.name.node,
                          field_doc);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  if field.count.node == 1 {
    let shift = utils::shift(cx, None, field);
    utils::unwrap_impl_item(quote_item!(cx,
      impl<'a> $setter_ty<'a> {
        $doc_attr
        pub fn $fn_name<'b>(&'b mut self) -> &'b mut $setter_ty<'a> {
          self.value |= $mask << $shift;
          self.mask |= $mask << $shift;
          self
        }
      }
    ).unwrap())
  } else {
    let shift = utils::shift(cx, Some(quote_expr!(cx, idx)), field);
    utils::unwrap_impl_item(quote_item!(cx,
      impl<'a> $setter_ty<'a> {
        $doc_attr
        pub fn $fn_name<'b>(&'b mut self, idx: usize) -> &'b mut $setter_ty<'a> {
          self.value |= $mask << $shift;
          self.mask |= $mask << $shift;
          self
        }
      }
    ).unwrap())
  }
}
