// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
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

#![feature(rustc_private, plugin_registrar, quote, convert)]

extern crate platformtree;
extern crate rustc;
extern crate serialize;
extern crate syntax;

use std::clone::Clone;
use std::ops::Deref;

use rustc::plugin::Registry;
use syntax::ast;
use syntax::codemap::DUMMY_SP;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, Modifier};
use syntax::ext::build::AstBuilder;
use syntax::owned_slice::OwnedSlice;
use syntax::print::pprust;
use syntax::util::small_vector::SmallVector;
use syntax::ptr::P;

use platformtree::parser::Parser;
use platformtree::builder::Builder;
use platformtree::builder::meta_args::ToTyHash;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("platformtree", macro_platformtree);
  reg.register_macro("platformtree_verbose", macro_platformtree_verbose);
  reg.register_syntax_extension(syntax::parse::token::intern("zinc_task"),
      Modifier(Box::new(macro_zinc_task)));
}

pub fn macro_platformtree(cx: &mut ExtCtxt, _: Span, tts: &[ast::TokenTree])
    -> Box<MacResult+'static> {
  let pt = Parser::new(cx, tts).parse_platformtree();
  let items = Builder::build(cx, pt.unwrap())
    .expect(format!("Unexpected failure on {}", line!()).as_str())
    .emit_items(cx);
  MacItems::new(items)
}

pub fn macro_platformtree_verbose(cx: &mut ExtCtxt, sp: Span,
    tts: &[ast::TokenTree]) -> Box<MacResult+'static> {
  let result = macro_platformtree(cx, sp, tts);
  println!("Platform Tree dump:");
  for i in result.make_items().unwrap().as_slice().iter() {
    println!("{}", pprust::item_to_string(i.deref()));
  }

  macro_platformtree(cx, sp, tts)
}

fn macro_zinc_task(cx: &mut ExtCtxt, _: Span, _: &ast::MetaItem,
    it: P<ast::Item>) -> P<ast::Item> {
  match it.node {
    ast::ItemFn(ref decl, style, abi, _, ref block) => {
      let istr = syntax::parse::token::get_ident(it.ident);
      let fn_name = &*istr;
      let ty_params = platformtree::builder::meta_args::get_ty_params_for_task(cx, fn_name);

      let params = ty_params.iter().map(|ty| {
        cx.typaram(
            DUMMY_SP,
            cx.ident_of(ty.to_tyhash().as_str()),
            OwnedSlice::from_vec(vec!(cx.typarambound(
                cx.path(DUMMY_SP, ty.as_str().split("::").map(|t| cx.ident_of(t)).collect())))),
            None)
      }).collect();

      let new_arg = cx.arg(DUMMY_SP, cx.ident_of("args"), cx.ty_rptr(
          DUMMY_SP,
          cx.ty_path(
              cx.path_all(
                  DUMMY_SP,
                  false,
                  ["pt".to_string(), fn_name.to_string() + "_args"].iter().map(|t| cx.ident_of(t.as_str())).collect(),
                  vec!(),
                  ty_params.iter().map(|ty| {
                    cx.ty_path(cx.path_ident(DUMMY_SP, cx.ident_of(ty.to_tyhash().as_str())))
                  }).collect(),
                  vec!())),
          None,
          ast::MutImmutable));
      let new_decl = P(ast::FnDecl {
        inputs: vec!(new_arg),
        ..decl.deref().clone()
      });

      let new_generics = ast::Generics {
        lifetimes: vec!(),
        ty_params: OwnedSlice::from_vec(params),
        where_clause: ast::WhereClause {
          id: ast::DUMMY_NODE_ID,
          predicates: vec!(),
        }
      };
      let new_node = ast::ItemFn(new_decl, style, abi, new_generics, block.clone());

      P(ast::Item {node: new_node, ..it.deref().clone() })
    },
    _ => panic!(),
  }
}

pub struct MacItems {
  items: Vec<P<ast::Item>>
}

impl MacItems {
  pub fn new(items: Vec<P<ast::Item>>) -> Box<MacResult+'static> {
    Box::new(MacItems { items: items })
  }
}
impl MacResult for MacItems {
  fn make_items(self: Box<MacItems>) -> Option<SmallVector<P<ast::Item>>> {
    Some(SmallVector::many(self.items.clone()))
  }
}
