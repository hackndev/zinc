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

use syntax::ext::base::ExtCtxt;
use syntax::ast;
use syntax::ast::P;
use syntax::codemap::DUMMY_SP;
use syntax::ext::build::AstBuilder;
use syntax::parse::token;

use super::super::node;

/// Generate an unsuffixed integer literal expression with a dummy span
pub fn expr_int<'a>(cx: &'a ExtCtxt, n: i64) -> P<ast::Expr> {
  let sign = if n < 0 {ast::Minus} else {ast::Plus};
  cx.expr_lit(DUMMY_SP, ast::LitInt(n as u64, ast::UnsuffixedIntLit(sign)))
}

/// The name of the structure representing a register
pub fn path_ident<'a>(cx: &'a ExtCtxt, path: &Vec<String>)
                      -> ast::Ident {
  cx.ident_of(path.clone().connect("_").as_slice())
}


/// Generate a `#[name(...)]` attribute of the given type
pub fn list_attribute<'a>(cx: &'a ExtCtxt, name: &'static str,
    list: Vec<&'static str>) -> ast::Attribute {
  let words =
   list.move_iter()
   .map(|word| cx.meta_word(DUMMY_SP, token::InternedString::new(word)));
  let allow = cx.meta_list(DUMMY_SP, token::InternedString::new(name),
                                FromIterator::from_iter(words));
  cx.attribute(DUMMY_SP, allow)
}

/// Generate a `#[doc="..."]` attribute of the given type
pub fn doc_attribute<'a>(cx: &'a ExtCtxt,
    docstring: token::InternedString) -> ast::Attribute {
  let s: ast::Lit_ = ast::LitStr(docstring, ast::CookedStr);
  let attr =
    cx.meta_name_value(DUMMY_SP, token::InternedString::new("doc"), s);
  cx.attribute(DUMMY_SP, attr)
}

pub fn primitive_type_path(cx: &ExtCtxt, width: node::RegWidth)
                       -> ast::Path {
  let name = match width {
    node::Reg8  => "u8",
    node::Reg16 => "u16",
    node::Reg32 => "u32",
  };
  cx.path_ident(DUMMY_SP, cx.ident_of(name))
}

/// The `Path` to the type corresponding to the primitive type of
/// the given register
pub fn reg_primitive_type_path(cx: &ExtCtxt, reg: &node::Reg)
                               -> Option<ast::Path> {
  match reg.ty {
    node::RegPrim(width, _) => Some(primitive_type_path(cx, width)),
    _ => None,
  }
}

pub fn reg_primitive_type(cx: &ExtCtxt, reg: &node::Reg)
                          -> Option<P<ast::Ty>> {
  let path = reg_primitive_type_path(cx, reg);
  path.map(|p| cx.ty_path(p, None))
}

pub fn field_type_path<'a>(cx: &'a ExtCtxt, path: &Vec<String>,
    reg: &node::Reg, field: &node::Field) -> ast::Path {
  let span = field.ty.span;
  match field.ty.node {
    node::UIntField => {
      match reg.ty {
        node::RegPrim(width, _) => primitive_type_path(cx, width),
        _  => fail!("The impossible happened: a union register with fields"),
      }
    },
    node::BoolField => cx.path_ident(span, cx.ident_of("bool")),
    node::EnumField { opt_name: ref opt_name, ..} => {
      match opt_name {
        &Some(ref name) =>
          cx.path_ident(span, cx.ident_of(name.as_slice())),
        &None => {
          let name =
            path.clone().append_one(field.name.node.clone()).connect("_");
          cx.path_ident(span, cx.ident_of(name.as_slice()))
        }
      }
    },
  }
}

/// Build an expression for the mask of a field
pub fn mask<'a>(cx: &'a ExtCtxt, field: &node::Field) -> P<ast::Expr> {
  expr_int(cx, ((1i << field.width) - 1) as i64)
}

/// Build an expression for the shift of a field (including the array
/// index if necessary)
pub fn shift<'a>(cx: &'a ExtCtxt, idx: Option<P<ast::Expr>>,
                 field: &node::Field) -> P<ast::Expr> {
  let low = expr_int(cx, field.low_bit as i64);
  match idx {
    Some(idx) => {
      let width = expr_int(cx, field.width as i64);
      quote_expr!(cx, $low + $idx * $width)
    },
    None => low,
  }
}

/// The name of the setter type for a register
pub fn setter_name<'a>(cx: &'a ExtCtxt, path: &Vec<String>) -> ast::Ident {
  let s = path.clone().append_one("Update".to_string());
  path_ident(cx, &s)
}

/// The name of the getter type for a register
pub fn getter_name<'a>(cx: &'a ExtCtxt, path: &Vec<String>) -> ast::Ident {
  let s = path.clone().append_one("Get".to_string());
  path_ident(cx, &s)
}

pub fn intern_string(cx: &ExtCtxt, s: String) -> token::InternedString {
  token::get_ident(cx.ident_of(s.as_slice()))
}
