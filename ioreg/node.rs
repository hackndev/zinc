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

use syntax::codemap::Span;

pub struct EnumValue {
  pub name: String,
  pub name_span: Span,
  pub value: uint,
  pub value_span: Span,
}

pub enum FieldType {
  /// A unsigned integer with given bit-width
  UIntType(uint),
  /// A enum with given width
  EnumType(Option<String>, Vec<EnumValue>, uint),
  /// A struct
  StructType(Option<String>, Vec<FieldOrPadding>),
}

pub struct Field {
  pub name: String,
  pub name_span: Span,

  pub read_only: bool,
  pub ty: FieldType,

  pub count: uint,
  pub count_span: Span,
}

pub enum FieldOrPadding {
  Field(Field),
  Padding(uint),
}

pub struct IoReg {
  pub name: String,
  pub name_span: Span,
  pub fields: Vec<FieldOrPadding>,
}
