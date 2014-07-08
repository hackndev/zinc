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
use serialize::{Encodable};

/// A variant of an enum field type
#[deriving(Clone, Decodable, Encodable)]
pub struct Variant {
  pub name: Spanned<String>,
  pub value: Spanned<uint>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

/// A bit field type
#[deriving(Clone, Decodable, Encodable)]
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

#[deriving(Clone, Decodable, Encodable)]
pub enum Access {
  ReadWrite,
  ReadOnly,
  WriteOnly,
}

#[deriving(Clone, Decodable, Encodable)]
pub struct Field {
  pub name: Spanned<String>,
  pub bits: Spanned<(uint, uint)>,
  pub access: Access,
  pub ty: Spanned<FieldType>,
  pub count: Spanned<uint>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

#[deriving(Clone, Decodable, Encodable)]
pub enum RegWidth {
  /// A 32-bit wide register
  Reg32,
  /// A 16-bit wide register
  Reg16,
  /// An 8-bit wide register
  Reg8,
}

impl RegWidth {
  /// Size of register type in bytes
  pub fn size(&self) -> uint {
    match *self {
      Reg32 => 4,
      Reg16 => 2,
      Reg8  => 8,
    }
  }
}

#[deriving(Clone, Decodable, Encodable)]
pub enum RegType {
  /// A primitive bitfield
  RegPrim(RegWidth, Vec<Field>),
  /// A group
  RegUnion(Gc<Vec<Reg>>),
}

impl RegType {
  /// Size of register type in bytes
  pub fn size(&self) -> uint {
    match *self {
      RegPrim(width, _) => width.size(),
      RegUnion(group)   => group.size(),
    }
  }
}

#[deriving(Clone, Decodable, Encodable)]
pub struct Reg {
  pub offset: uint,
  pub name: Spanned<String>,
  pub ty: RegType,
  pub count: Spanned<uint>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

impl Reg {
  /// Size of a register in bytes
  pub fn size(&self) -> uint {
    self.count.node * self.ty.size()
  }
}

#[deriving(Clone, Decodable, Encodable)]
pub struct RegGroup {
  pub name: Spanned<String>,
  pub docstring: Option<Spanned<ast::Ident>>,
  pub regs: Vec<Reg>,
}

impl Vec<Reg> {
  /// Size of registers of register group in bytes
  pub fn size(&self) -> uint {
    match self.iter().max_by(|r| r.offset) {
      Some(last) => last.offset + last.ty.size(),
      None => 0,
    }
  }
}
