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

use std::gc::{Gc, GC};
use syntax::ast;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

use node;
mod utils;

mod setter;
mod getter;
mod union;
mod register;
mod accessors;

pub struct Builder {
  items: Vec<P<ast::Item>>,
}

impl Builder {
  pub fn new() -> Builder {
    Builder {items: Vec::new()}
  }

  pub fn emit_items(&mut self, cx: &ExtCtxt, reg: P<node::Reg>)
                    -> Vec<Gc<ast::Item>> {
    node::visit_reg(&*reg, &mut setter::BuildSetters::new(self, cx));
    node::visit_reg(&*reg, &mut getter::BuildGetters::new(self, cx));
    node::visit_reg(&*reg, &mut register::BuildRegStructs::new(self, cx));
    node::visit_reg(&*reg, &mut union::BuildUnionTypes::new(self, cx));
    node::visit_reg(&*reg, &mut accessors::BuildAccessors::new(self, cx));

    let cloned = self.items.clone();
    let mut regc = vec!();
    for ref c in cloned.iter() {
      let ritem: &ast::Item = c.deref();
      let item: ast::Item = ritem.clone();
      regc.push(box(GC) item);
    }
    regc
  }

  pub fn push_item(&mut self, item: P<ast::Item>) {
    self.items.push(item);
  }
}

