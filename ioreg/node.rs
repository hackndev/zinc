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

use syntax::codemap::{Spanned};
use syntax::ast;
use std::collections::hashmap::HashMap;
use std::gc::Gc;

/// A variant of an enum field type
#[deriving(Clone)]
pub struct Variant {
  pub name: Spanned<String>,
  pub value: Spanned<uint>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

/// A bit field type
#[deriving(Clone)]
pub enum FieldType {
  /// A unsigned integer with given bit-width
  UIntField,
  /// A boolean flag
  BoolField,
  /// A enum
  EnumField {
    pub opt_name: Option<String>,
    pub variants: Vec<Variant>,
  },
}

#[deriving(Clone)]
pub enum Access {
  ReadWrite,
  ReadOnly,
  WriteOnly,
}

#[deriving(Clone)]
pub struct Field {
  pub name: Spanned<String>,
  pub bits: Spanned<(uint, uint)>,
  pub access: Access,
  pub ty: Spanned<FieldType>,
  pub count: Spanned<uint>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

#[deriving(Clone)]
pub enum RegType {
  /// A 32-bit wide register
  U32Reg,
  /// A 16-bit wide register
  U16Reg,
  /// An 8-bit wide register
  U8Reg,
  /// A group
  GroupReg(Gc<RegGroup>),
}

impl RegType {
  /// Size of register type in bytes
  pub fn size(&self) -> uint {
    match *self {
      U32Reg => 4,
      U16Reg => 2,
      U8Reg  => 8,
      GroupReg(group) => group.size(),
    }
  }
}

#[deriving(Clone)]
pub struct Reg {
  pub offset: uint,
  pub name: Spanned<String>,
  pub ty: RegType,
  pub count: Spanned<uint>,
  pub fields: Vec<Field>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

impl Reg {
  /// Size of a register in bytes
  pub fn size(&self) -> uint {
    self.count.node * self.ty.size()
  }
}

pub struct RegGroup {
  pub name: Spanned<String>,
  pub regs: Vec<Reg>,
  pub groups: HashMap<String, Gc<RegGroup>>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

impl RegGroup {
  /// Size of registers of register group in bytes
  pub fn size(&self) -> uint {
    match self.regs.iter().max_by(|r| r.offset) {
      Some(last) => last.offset + last.ty.size(),
      None => 0,
    }
  }
}
