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

use syntax::ext::base::ExtCtxt;
use syntax::ast;
use syntax::ptr::P;
use syntax::codemap::{respan, Span, Spanned};
use syntax::ext::build::AstBuilder;
use syntax::parse::token;

use super::super::node;

/// Generate an unsuffixed integer literal expression with a dummy span
pub fn expr_int(cx: &ExtCtxt, n: Spanned<i64>) -> P<ast::Expr> {
  let sign = if n.node < 0 {ast::Minus} else {ast::Plus};
  cx.expr_lit(n.span, ast::LitInt(n.node as u64, ast::UnsuffixedIntLit(sign)))
}

/// The name of the structure representing a register
pub fn path_ident(cx: &ExtCtxt, path: &Vec<String>)
                      -> ast::Ident {
  cx.ident_of(path.clone().connect("_").as_str())
}


/// Generate a `#[name(...)]` attribute of the given type
pub fn list_attribute(cx: &ExtCtxt, name: &'static str,
                      list: Vec<&'static str>,
                      span: Span) -> ast::Attribute {
  let spanned_name = respan(span, name);
  let spanned_list: Vec<Spanned<&'static str>> = list.into_iter()
    .map(|word| respan(span, word))
    .collect();
  list_attribute_spanned(cx, spanned_name, spanned_list)
}

fn list_attribute_spanned(cx: &ExtCtxt, name: Spanned<&'static str>,
    list: Vec<Spanned<&'static str>>) -> ast::Attribute {
  let words =
   list.into_iter()
   .map(|word| cx.meta_word(word.span, token::InternedString::new(word.node)));
  let allow = cx.meta_list(name.span, token::InternedString::new(name.node),
                                FromIterator::from_iter(words));
  cx.attribute(name.span, allow)
}

/// Generate a `#[doc="..."]` attribute of the given type
#[allow(dummy_span)]
pub fn doc_attribute(cx: &ExtCtxt, docstring: token::InternedString)
                     -> ast::Attribute {
  use syntax::codemap::DUMMY_SP;

  let s: ast::Lit_ = ast::LitStr(docstring, ast::CookedStr);
  let attr =
    cx.meta_name_value(DUMMY_SP, token::InternedString::new("doc"), s);
  cx.attribute(DUMMY_SP, attr)
}

pub fn primitive_type_path(cx: &ExtCtxt, width: &Spanned<node::RegWidth>)
                           -> ast::Path {
  let name = match width.node {
    node::RegWidth::Reg8  => "u8",
    node::RegWidth::Reg16 => "u16",
    node::RegWidth::Reg32 => "u32",
  };
  cx.path_ident(width.span, cx.ident_of(name))
}

/// The `Path` to the type corresponding to the primitive type of
/// the given register
pub fn reg_primitive_type_path(cx: &ExtCtxt, reg: &node::Reg)
                               -> Option<ast::Path> {
  match reg.ty {
    node::RegType::RegPrim(ref width, _) => Some(primitive_type_path(cx, width)),
    _ => None,
  }
}

pub fn reg_primitive_type(cx: &ExtCtxt, reg: &node::Reg)
                          -> Option<P<ast::Ty>> {
  let path = reg_primitive_type_path(cx, reg);
  path.map(|p| cx.ty_path(p))
}

pub fn field_type_path(cx: &ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, field: &node::Field) -> ast::Path {
  let span = field.ty.span;
  match field.ty.node {
    node::FieldType::UIntField => {
      match reg.ty {
        node::RegType::RegPrim(ref width, _) => primitive_type_path(cx,
                                                                    width),
        _  => panic!("The impossible happened: a union register with fields"),
      }
    },
    node::FieldType::BoolField => cx.path_ident(span, cx.ident_of("bool")),
    node::FieldType::EnumField { ref opt_name, ..} => {
      match opt_name {
        &Some(ref name) =>
          cx.path_ident(span, cx.ident_of(name.as_str())),
        &None => {
          let mut name = path.clone();
          name.push(field.name.node.clone());
          cx.path_ident(span, cx.ident_of(name.connect("_").as_str()))
        }
      }
    },
  }
}

pub fn unwrap_impl_item(item: P<ast::Item>) -> P<ast::ImplItem> {
  match item.node {
    ast::ItemImpl(_, _, _, _, _, ref items) => {
      items.clone().pop().expect("ImplItem not found")
    },
    _ => panic!("Tried to unwrap ImplItem from Non-Impl")
  }
}

/// Build an expression for the mask of a field
pub fn mask(cx: &ExtCtxt, field: &node::Field) -> P<ast::Expr> {
  expr_int(cx, respan(field.bit_range_span,
                      ((1 << field.width as u64) - 1)))
}

/// Build an expression for the shift of a field (including the array
/// index if necessary)
pub fn shift(cx: &ExtCtxt, idx: Option<P<ast::Expr>>,
                 field: &node::Field) -> P<ast::Expr> {
  let low = expr_int(cx, respan(field.bit_range_span, field.low_bit as i64));
  match idx {
    Some(idx) => {
      let width = expr_int(cx, respan(field.bit_range_span,
                                      field.width as i64));
      quote_expr!(cx, $low + $idx * $width)
    },
    None => low,
  }
}

/// The name of the setter type for a register
pub fn setter_name(cx: &ExtCtxt, path: &Vec<String>) -> ast::Ident {
  let mut s = path.clone();
  s.push("Update".to_string());
  path_ident(cx, &s)
}

/// The name of the getter type for a register
pub fn getter_name(cx: &ExtCtxt, path: &Vec<String>) -> ast::Ident {
  let mut s = path.clone();
  s.push("Get".to_string());
  path_ident(cx, &s)
}

pub fn intern_string(cx: &ExtCtxt, s: String) -> token::InternedString {
  cx.ident_of(s.as_str()).name.as_str()
}
