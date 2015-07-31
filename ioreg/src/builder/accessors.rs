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
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;

use super::Builder;
use super::utils;
use super::super::node;

/// A visitor to build accessor functions for each register struct
pub struct BuildAccessors<'a> {
  builder: &'a mut Builder,
  cx: &'a ExtCtxt<'a>,
}

impl<'a> node::RegVisitor for BuildAccessors<'a> {
  fn visit_prim_reg(&mut self, path: &Vec<String>, reg: &node::Reg,
                    fields: &Vec<node::Field>) {
    if fields.iter().any(|f| f.access != node::Access::WriteOnly) {
      let item = build_get_fn(self.cx, path, reg);
      self.builder.push_item(item);
    }
    if fields.iter().any(|f| f.access != node::Access::ReadOnly) {
      let item = build_ignoring_state_setter_fn(self.cx, path, reg);
      self.builder.push_item(item);
    }

    for field in fields.iter() {
      match build_field_accessors(self.cx, path, reg, field) {
        Some(item) => self.builder.push_item(item),
        None       => {}
      }
    }
  }
}

impl<'a> BuildAccessors<'a> {
  pub fn new(builder: &'a mut Builder, cx: &'a ExtCtxt<'a>)
             -> BuildAccessors<'a> {
    BuildAccessors {builder: builder, cx: cx}
  }
}

fn build_field_accessors(cx: &ExtCtxt, path: &Vec<String>,
                         reg: &node::Reg, field: &node::Field)
                         -> Option<P<ast::Item>>
{
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(reg.name.span, utils::path_ident(cx, path));

  let items = match field.access {
    node::Access::ReadWrite => vec!(build_field_set_fn(cx, path, reg, field),
                            build_field_get_fn(cx, path, reg, field)),
    node::Access::ReadOnly  => vec!(build_field_get_fn(cx, path, reg, field)),
    node::Access::WriteOnly => vec!(build_field_set_fn(cx, path, reg, field)),
    node::Access::SetToClear => vec!(build_field_clear_fn(cx, path, reg, field),
                             build_field_get_fn(cx, path, reg, field)),
  };

  let access_tag = match field.access {
    node::Access::ReadWrite => "read/write",
    node::Access::ReadOnly  => "read-only",
    node::Access::WriteOnly => "write-only",
    node::Access::SetToClear => "set-to-clear",
  };

  let field_doc = match field.docstring {
    Some(ref d) => {
      let s = d.node.name.as_str();
      s.to_string()
    },
    None => "no documentation".to_string()
  };
  let docstring = format!("*[{}]* `{}` field: {}",
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

fn build_get_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg)
                -> P<ast::Item>
{
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let getter_ty = utils::getter_name(cx, path);

  let docstring = format!("Fetch the value of the `{}` register",
                          reg.name.node);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let item = quote_item!(cx,
    impl $reg_ty {
      $doc_attr
      #[allow(dead_code)]
      pub fn get(&self) -> $getter_ty {
        $getter_ty::new(self)
      }
    }
    );
  item.unwrap()
}

fn build_ignoring_state_setter_fn(cx: &ExtCtxt, path: &Vec<String>, reg: &node::Reg)
                -> P<ast::Item>
{
  let reg_ty: P<ast::Ty> =
    cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let setter_ty = utils::setter_name(cx, path);

  let docstring = format!("Create new updater that ignores current value of the `{}` register",
                          reg.name.node);
  let doc_attr = utils::doc_attribute(cx, utils::intern_string(cx, docstring));

  let item = quote_item!(cx,
    impl $reg_ty {
      $doc_attr
      #[allow(dead_code)]
      pub fn ignoring_state(&self) -> $setter_ty {
        $setter_ty::new_ignoring_state(self)
      }
    }
    );
  item.unwrap()
}

fn build_field_set_fn(cx: &ExtCtxt, path: &Vec<String>,
                      reg: &node::Reg, field: &node::Field)
                      -> P<ast::ImplItem>
{
  let reg_ty = cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let fn_name =
    cx.ident_of((String::from("set_")+field.name.node.as_str()).as_str());
  let field_ty: P<ast::Ty> =
    cx.ty_path(utils::field_type_path(cx, path, reg, field));
  let setter_ty = utils::setter_name(cx, path);
  if field.count.node == 1 {
    utils::unwrap_impl_item(quote_item!(cx,
      impl $reg_ty {
        #[allow(dead_code, missing_docs)]
        pub fn $fn_name<'a>(&'a self, new_value: $field_ty) -> $setter_ty<'a> {
          let mut setter: $setter_ty = $setter_ty::new(self);
          setter.$fn_name(new_value);
          setter
        }
      }
    ).unwrap())
  } else {
    utils::unwrap_impl_item(quote_item!(cx,
      impl $reg_ty {
        #[allow(dead_code, missing_docs)]
        pub fn $fn_name<'a>(&'a self, idx: usize, new_value: $field_ty) -> $setter_ty<'a> {
          let mut setter: $setter_ty = $setter_ty::new(self);
          setter.$fn_name(idx, new_value);
          setter
        }
      }
    ).unwrap())
  }
}

fn build_field_get_fn(cx: &ExtCtxt, path: &Vec<String>,
                      reg: &node::Reg, field: &node::Field)
                      -> P<ast::ImplItem>
{
  let reg_ty = cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  utils::unwrap_impl_item({
      let fn_name = cx.ident_of(field.name.node.as_str());
      let field_ty: P<ast::Ty> =
        cx.ty_path(utils::field_type_path(cx, path, reg, field));
      let getter_ty = utils::getter_name(cx, path);
      if field.count.node == 1 {
        quote_item!(cx,
          impl $reg_ty {
            #[allow(dead_code, missing_docs)]
            pub fn $fn_name(&self) -> $field_ty {
              $getter_ty::new(self).$fn_name()
            }
          }
        ).unwrap()
      } else {
        quote_item!(cx,
          impl $reg_ty {
            #[allow(dead_code, missing_docs)]
            pub fn $fn_name(&self, idx: usize) -> $field_ty {
              $getter_ty::new(self).$fn_name(idx)
            }
          }
        ).unwrap()
      }
  })
}

fn build_field_clear_fn(cx: &ExtCtxt, path: &Vec<String>,
                        reg: &node::Reg, field: &node::Field)
                        -> P<ast::ImplItem>
{
  let reg_ty = cx.ty_ident(reg.name.span, utils::path_ident(cx, path));
  let fn_name =
    cx.ident_of((String::from("clear_")+field.name.node.as_str()).as_str());
  let setter_ty = utils::setter_name(cx, path);
  utils::unwrap_impl_item(if field.count.node == 1 {
    quote_item!(cx,
      impl $reg_ty {
        #[allow(dead_code, missing_docs)]
        pub fn $fn_name<'a>(&'a self) -> $setter_ty<'a> {
          let mut setter: $setter_ty = $setter_ty::new(self);
          setter.$fn_name();
          setter
        }
      }
    ).unwrap()
  } else {
    quote_item!(cx,
      impl $reg_ty {
        #[allow(dead_code, missing_docs)]
        pub fn $fn_name<'a>(&'a self, idx: usize) -> $setter_ty<'a> {
          let mut setter: $setter_ty = $setter_ty::new(self);
          setter.$fn_name(idx);
          setter
        }
      }
    ).unwrap()
  })
}
