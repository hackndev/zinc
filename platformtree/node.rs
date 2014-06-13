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

use syntax::codemap::{DUMMY_SP, Span};
use std::collections::hashmap;
use std::gc::Gc;

#[deriving(Show, PartialEq)]
pub enum AttributeValue {
  UIntValue(uint),
  StrValue(String),
  RefValue(String),
}

#[deriving(Show)]
pub struct Path {
  pub absolute: bool,
  pub path: Vec<String>,
  pub span: Option<Span>,
}

impl Path {
  pub fn new(absolute: bool, path: Vec<&str>) -> Path {
    Path {
      absolute: absolute,
      path: path.iter().map(|x| x.to_string()).collect(),
      span: Some(DUMMY_SP),
    }
  }
}

/// Tree node.
#[deriving(Show)]
pub struct Node {
  /// Node name.
  pub name: Option<String>,

  /// Node path.
  pub path: Path,

  /// Node attributes.
  pub attributes: hashmap::HashMap<String, AttributeValue>,

  /// Child nodes.
  pub subnodes: Vec<Gc<Node>>,
}

impl Node {
  pub fn new() -> Node {
    Node {
      name: None,
      path: Path {
        absolute: false,
        path: Vec::new(),
        span: Some(DUMMY_SP),
      },
      attributes: hashmap::HashMap::new(),
      subnodes: Vec::new(),
    }
  }
}
