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

use syntax::codemap::{Span, Spanned};
use syntax::ast;
use std::collections::hashmap::HashMap;
use std::gc::Gc;

pub struct EnumValue {
  pub name: Spanned<String>,
  pub value: Spanned<uint>,
}

pub enum FieldType {
  /// A unsigned integer with given bit-width
  UIntField,
  /// A boolean flag
  BoolField,
  /// A enum
  EnumField(Option<String>, Vec<EnumValue>),
}

pub enum Access {
  ReadWrite,
  ReadOnly,
  WriteOnly,
}

pub struct Field {
  pub name: Spanned<String>,
  pub bits: Spanned<(uint, uint)>,
  pub access: Access,
  pub ty: Spanned<FieldType>,
  pub count: Spanned<uint>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

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

pub struct Reg {
  pub offset: uint,
  pub name: Spanned<String>,
  pub ty: RegType,
  pub count: Spanned<uint>,
  pub fields: Vec<Field>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

pub struct RegGroup {
  pub name: Spanned<String>,
  pub regs: Vec<Reg>,
  pub groups: HashMap<String, Gc<RegGroup>>,
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
