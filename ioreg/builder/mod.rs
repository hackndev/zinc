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

use std::gc::Gc;
use syntax::ast;
use syntax::ast::P;
use syntax::ext::base::ExtCtxt;

use node;
mod utils;

mod setter;
mod getter;
mod union;
mod register;
mod accessors;

pub struct Builder {
  items: Vec<Gc<ast::Item>>,
}

impl Builder {
  pub fn new() -> Builder {
    Builder {items: Vec::new()}
  }

  pub fn emit_items<'a>(&mut self, cx: &'a ExtCtxt, reg: Gc<node::Reg>)
                    -> Vec<P<ast::Item>> {
    node::visit_reg(&*reg, &mut setter::BuildSetters::new(self, cx));
    node::visit_reg(&*reg, &mut getter::BuildGetters::new(self, cx));
    node::visit_reg(&*reg, &mut register::BuildRegStructs::new(self, cx));
    node::visit_reg(&*reg, &mut union::BuildUnionTypes::new(self, cx));
    node::visit_reg(&*reg, &mut accessors::BuildAccessors::new(self, cx));
    self.items.clone()
  }

  pub fn push_item(&mut self, item: Gc<ast::Item>) {
    self.items.push(item);
  }
}

