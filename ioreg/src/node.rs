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

use std::ops::Deref;
use std::rc::Rc;
use syntax::codemap::{Spanned, Span};
use syntax::ast;

/// A variant of an enum field type
#[derive(Clone)]
pub struct Variant {
  pub name: Spanned<String>,
  pub value: Spanned<u64>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

/// A bit field type
#[derive(Clone)]
pub enum FieldType {
  /// A unsigned integer
  UIntField,
  /// A boolean flag
  BoolField,
  /// A enum
  EnumField {
    opt_name: Option<String>,
    variants: Vec<Variant>,
  },
}

#[derive(Copy, PartialEq, Eq, Clone)]
pub enum Access {
  ReadWrite,
  ReadOnly,
  WriteOnly,
  /// A flag which can be set to clear
  SetToClear,
}

#[derive(Clone)]
pub struct Field {
  pub name: Spanned<String>,
  /// The index of the first (lowest order) bit of the field
  pub low_bit: u8,
  /// The width in bits of a single array element
  pub width: u8,
  /// The number of array elements
  pub count: Spanned<u8>,
  pub bit_range_span: Span,
  pub access: Access,
  pub ty: Spanned<FieldType>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

impl Field {
  /// The index of the highest order bit owned by this field
  pub fn high_bit(&self) -> u8 {
    self.low_bit + self.width * self.count.node - 1
  }
}

#[derive(Copy, Clone)]
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
  pub fn size(&self) -> u64 {
    match *self {
      RegWidth::Reg32 => 4,
      RegWidth::Reg16 => 2,
      RegWidth::Reg8  => 1,
    }
  }
}

#[derive(Clone)]
pub enum RegType {
  /// A primitive bitfield
  RegPrim(RegWidth, Vec<Field>),
  /// A group
  RegUnion(Rc<Vec<Reg>>),
}

impl RegType {
  /// Size of register type in bytes
  pub fn size(&self) -> u64 {
    match self {
      &RegType::RegPrim(ref width, _)  => width.size() as u64,
      &RegType::RegUnion(ref regs) => regs_size(regs.deref()),
    }
  }
}

/// A single register, either a union or primitive
#[derive(Clone)]
pub struct Reg {
  pub offset: u64,
  pub name: Spanned<String>,
  pub ty: RegType,
  pub count: Spanned<u32>,
  pub docstring: Option<Spanned<ast::Ident>>,
}

impl Reg {
  /// Size of a register in bytes
  pub fn size(&self) -> u64 {
    self.count.node as u64 * self.ty.size()
  }
  /// The offset of the last byte owned by this register
  pub fn last_byte(&self) -> u64 {
    self.offset + self.size() - 1
  }
}

/// Size of registers of register group in bytes
pub fn regs_size(regs: &Vec<Reg>) -> u64 {
  match regs.iter().max_by(|r| r.offset) {
    Some(last) => last.offset + last.ty.size(),
    None => 0,
  }
}

pub trait RegVisitor {
  /// Path includes name of `Reg` being visited
  fn visit_prim_reg<'a>(&'a mut self, _path: &Vec<String>, _reg: &'a Reg,
                        _width: &RegWidth, _fields: &Vec<Field>) {}
  fn visit_union_reg<'a>(&'a mut self, _path: &Vec<String>, _reg: &'a Reg,
                         _subregs: Rc<Vec<Reg>>) {}
}

pub fn visit_reg<T: RegVisitor>(reg: &Reg, visitor: &mut T) {
  visit_reg_(reg, visitor, vec!(reg.name.node.clone()))
}

fn visit_reg_<T: RegVisitor>(reg: &Reg, visitor: &mut T, path: Vec<String>) {
  match reg.ty {
    RegType::RegUnion(ref regs) => {
      visitor.visit_union_reg(&path, reg, regs.clone());
      for r in regs.iter() {
        let mut new_path = path.clone();
        new_path.push(r.name.node.clone()); // TODO(bgamari) fix clone
        visit_reg_(r, visitor, new_path);
      }
    },
    RegType::RegPrim(ref width, ref fields) =>
      visitor.visit_prim_reg(&path, reg, width, fields)
  }
}
