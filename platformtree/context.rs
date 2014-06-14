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

use std::gc::Gc;
use syntax::ast;
use syntax::codemap::{mk_sp, BytePos};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;

use node;

#[path="../src/hal/lpc17xx/platformtree.rs"] mod lpc17xx_pt;

pub struct PlatformContext {
  mod_items: Vec<Gc<ast::Item>>,
  main_stmts: Vec<Gc<ast::Stmt>>,
}

impl PlatformContext {
  pub fn new<'a>() -> PlatformContext {
    PlatformContext {
      mod_items: Vec::new(),
      main_stmts: Vec::new(),
    }
  }

  pub fn add_item(&mut self, item: Gc<ast::Item>) {
    self.mod_items.push(item);
  }

  pub fn add_main_statement(&mut self, stmt: Gc<ast::Stmt>) {
    self.main_stmts.push(stmt);
  }

  pub fn get_main_block(&self, ecx: &ExtCtxt) -> ast::P<ast::Block> {
    let sp = mk_sp(BytePos(0), BytePos(0));
    ecx.block(sp, self.main_stmts.clone(), None)
  }
}

pub fn process_node(pcx: &mut PlatformContext, ecx: &ExtCtxt, node: Gc<node::Node>) {
  match node.path.path.get(0).as_slice() {
    "mcu" => process_mcu(pcx, ecx, node.path.path.get(1), node),
    other => ecx.span_err(
        node.path.span.unwrap(),
        format!("unknown root path `{}`", other).as_slice()),
  }
}

fn process_mcu(pcx: &mut PlatformContext, ecx: &ExtCtxt, mcu: &String, node: Gc<node::Node>) {
  match mcu.as_slice() {
    "lpc17xx" => lpc17xx_pt::process_nodes(pcx, ecx, &node.subnodes),
    other => ecx.span_err(
        node.path.span.unwrap(),
        format!("unknown mcu `{}`", other).as_slice()),
  }
}

impl node::Node {
  pub fn unwrap_string(&self, ecx: &ExtCtxt, attr: &str) -> Option<String> {
    match self.unwrap_attribute(ecx, attr) {
      Some(a) => match a {
        node::StrValue(v) => Some(v),
        _ => {
          ecx.span_err(
              self.path.span.unwrap(),  // TODO: wrong span
              format!("required string key `{}` is not of a string type", attr).as_slice());
          None
        }
      },
      None => None,
    }
  }

  pub fn unwrap_int(&self, ecx: &ExtCtxt, attr: &str) -> Option<uint> {
    match self.unwrap_attribute(ecx, attr) {
      Some(a) => match a {
        node::UIntValue(v) => Some(v),
        _ => {
          ecx.span_err(
              self.path.span.unwrap(),  // TODO: wrong span
              format!("required string key `{}` is not of a string type", attr).as_slice());
          None
        }
      },
      None => None,
    }
  }

  fn unwrap_attribute(&self, ecx: &ExtCtxt, attr: &str) -> Option<node::AttributeValue> {
    match self.attributes.find_equiv(&attr.to_str()) {
      Some(a) => Some(a.clone()),
      None => {
        ecx.span_err(
            self.path.span.unwrap(),  // TODO: wrong span
            format!("required attribute `{}` is missing", attr).as_slice());
        None
      }
    }
  }
}
