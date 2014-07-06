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
use std::gc::Gc;

pub struct EnumValue {
  pub name: String,
  pub name_span: Span,
  pub value: uint,
  pub value_span: Span,
}

pub enum FieldType {
  /// A unsigned integer with given bit-width
  UIntType,
  /// A boolean flag
  BoolType,
  /// A enum
  EnumType(Option<String>, Vec<EnumValue>),
  /// A group specified by name
  GroupType(String),
}

pub struct Field {
  pub name: Spanned<String>,
  pub bits: Spanned<(uint, uint)>,
  pub read_only: bool,
  pub ty: Spanned<FieldType>,
  pub count: Spanned<uint>,
  pub docstring: Option<Spanned<String>>,
}

pub enum RegType {
  /// A unsigned integer with given bit-width
  UIntReg(uint),
  /// A group specified by name
  GroupReg(String),
}

pub struct Reg {
  pub name: String,
  pub name_span: Span,
  pub ty: RegType,
  pub count: Spanned<uint>,
  pub fields: Vec<Field>,
  pub docstring: Option<Spanned<String>>,
}

pub struct RegGroup {
  pub name: String,
  pub name_span: Span,
  pub regs: Vec<Reg>,
}
