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

use std::cell::{Cell, RefCell};
use std::collections::hashmap::HashMap;
use std::gc::Gc;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;

/// Holds a value for an attribute.
///
/// The value can be an unsigned integer, string or reference.
#[deriving(Clone)]
pub enum AttributeValue {
  IntValue(uint),
  StrValue(String),
  RefValue(String),
}

/// Expected attribute type.
///
/// Used in Node::expect_attributes to provide the expected type of the
/// attribute.
pub enum AttributeType {
  IntAttribute,
  StrAttribute,
  RefAttribute,
}

/// Attribute value and metadata.
///
/// Stored inside of a HashMap, the key to HashMap is the attribute name.
/// Provides spans for both key and value.
#[deriving(Clone)]
pub struct Attribute {
  pub value: AttributeValue,
  pub key_span: Span,
  pub value_span: Span,
}

impl Attribute {
  pub fn new(value: AttributeValue, key_span: Span, value_span: Span)
      -> Attribute {
    Attribute {
      value: value,
      key_span: key_span,
      value_span: value_span,
    }
  }

  pub fn new_nosp(value: AttributeValue) -> Attribute {
    Attribute {
      value: value,
      key_span: DUMMY_SP,
      value_span: DUMMY_SP,
    }
  }
}

/// PlatformTree node.
///
/// Might have an optional name, is the name is missing, name_span is equal to
/// path_span. Attributes are stored by name, subnodes are stored by path.
/// Type_name, if present, must specify the type path for the node's
/// materialized object.
pub struct Node {
  pub name: Option<String>,
  pub name_span: Span,

  pub path: String,
  pub path_span: Span,

  pub attributes: RefCell<HashMap<String, Attribute>>,
  pub subnodes: HashMap<String, Gc<Node>>,

  pub type_name: Cell<Option<&'static str>>,
}

impl Node {
  pub fn new(name: Option<String>, name_span: Span, path: String,
      path_span: Span) -> Node {
    Node {
      name: name,
      name_span: name_span,
      path: path,
      path_span: path_span,
      attributes: RefCell::new(HashMap::new()),
      subnodes: HashMap::new(),
      type_name: Cell::new(None),
    }
  }

  /// Returns attribute by name or fail!()s.
  pub fn get_attr(&self, key: &str) -> Attribute {
    self.attributes.borrow().get(&key.to_str()).clone()
  }

  /// Returns a string attribute by name or None, if it's not present or not of
  /// a StrAttribute type.
  pub fn get_string_attr(&self, key: &str) -> Option<String> {
    self.attributes.borrow().find(&key.to_str()).and_then(|av| match av.value {
      StrValue(ref s) => Some(s.clone()),
      _ => None,
    })
  }

  /// Returns an integer attribute by name or None, if it's not present or not
  /// of an IntAttribute type.
  pub fn get_int_attr(&self, key: &str) -> Option<uint> {
    self.attributes.borrow().find(&key.to_str()).and_then(|av| match av.value {
      IntValue(ref u) => Some(*u),
      _ => None,
    })
  }

  /// Returns a reference attribute by name or None, if it's not present or not
  /// of a RefAttribute type.
  pub fn get_ref_attr(&self, key: &str) -> Option<String> {
    self.attributes.borrow().find(&key.to_str()).and_then(|av| match av.value {
      RefValue(ref s) => Some(s.clone()),
      _ => None,
    })
  }

  /// Returns a string attribute by name or None, if it's not present or not of
  /// a StrAttribute type. Reports a parser error if an attribute is
  /// missing.
  pub fn get_required_string_attr(&self, cx: &ExtCtxt, key: &str)
      -> Option<String> {
    match self.get_string_attr(key) {
      Some(val) => Some(val),
      None => {
        cx.parse_sess().span_diagnostic.span_err(self.name_span,
            format!("required string attribute `{}` is missing", key)
            .as_slice());
        None
      }
    }
  }

  /// Returns an integer attribute by name or None, if it's not present or not
  /// of an IntAttribute type. Reports a parser error if an attribute is
  /// missing.
  pub fn get_required_int_attr(&self, cx: &ExtCtxt, key: &str)
      -> Option<uint> {
    match self.get_int_attr(key) {
      Some(val) => Some(val),
      None => {
        cx.parse_sess().span_diagnostic.span_err(self.name_span,
            format!("required integer attribute `{}` is missing", key)
            .as_slice());
        None
      }
    }
  }

  /// Returns a reference attribute by name or None, if it's not present or not
  /// of a RefAttribute type. Reports a parser error if an attribute is
  /// missing.
  pub fn get_required_ref_attr(&self, cx: &ExtCtxt, key: &str)
      -> Option<String> {
    match self.get_ref_attr(key) {
      Some(val) => Some(val),
      None => {
        cx.parse_sess().span_diagnostic.span_err(self.name_span,
            format!("required ref attribute `{}` is missing", key)
            .as_slice());
        None
      }
    }
  }

  /// Returns true if node has no attributes. Returs false and reports a parser
  /// error for each found attribute otherwise.
  pub fn expect_no_attributes(&self, cx: &ExtCtxt) -> bool {
    let mut ok = true;
    for (_, v) in self.attributes.borrow().iter() {
      ok = false;
      cx.parse_sess().span_diagnostic.span_err(v.key_span,
          "no attributes expected");
    }
    ok
  }

  /// Returns true if node has no subnodes. Returs false and reports a parser
  /// error for each found subnode otherwise.
  pub fn expect_no_subnodes(&self, cx: &ExtCtxt) -> bool {
    let mut ok = true;
    for (_, sub) in self.subnodes.iter() {
      ok = false;
      cx.parse_sess().span_diagnostic.span_err(sub.name_span,
          "no subnodes expected");
    }
    ok
  }

  /// Returns true if node has all of the requested attributes and their types
  /// match. Reports parser errors and returns false otherwise.
  pub fn expect_attributes(&self, cx: &ExtCtxt,
      expectations: &[(&str, AttributeType)]) -> bool {
    let mut ok = true;
    for &(n, ref t) in expectations.iter() {
      match t {
        &StrAttribute => {
          if self.get_required_string_attr(cx, n).is_none() {ok = false}
        },
        &IntAttribute => {
          if self.get_required_int_attr(cx, n).is_none() {ok = false}
        },
        &RefAttribute => {
          if self.get_required_ref_attr(cx, n).is_none() {ok = false}
        },
      }
    }
    ok
  }

  /// Returns true if node has all of the requested subnodes matched by path.
  /// Reports parser errors and returns false otherwise.
  pub fn expect_subnodes(&self, cx: &ExtCtxt, expectations: &[&str]) -> bool {
    let mut ok = true;
    for (path, sub) in self.subnodes.iter() {
      if !expectations.contains(&path.as_slice()) {
        ok = false;
        cx.parse_sess().span_diagnostic.span_err(sub.path_span,
            format!("unknown subnode `{}` in node `{}`",
                path, self.path).as_slice());
      }
    }
    ok
  }

  /// Returns a subnode by path or None, if not found.
  pub fn get_by_path<'a>(&'a self, path: &str) -> Option<&'a Gc<Node>> {
    self.subnodes.find(&path.to_str())
  }
}

/// PlatformTree root object.
///
/// Root nodes are stored by path in `nodes`, All the nmaed nodes are also
/// stored by name in `named`.
pub struct PlatformTree {
  nodes: HashMap<String, Gc<Node>>,
  named: HashMap<String, Gc<Node>>,
}

impl PlatformTree {
  pub fn new(nodes: HashMap<String, Gc<Node>>, named: HashMap<String, Gc<Node>>)
      -> PlatformTree {
    PlatformTree {
      nodes: nodes,
      named: named,
    }
  }

  /// Returns a node by name or None, if not found.
  pub fn get_by_name<'a>(&'a self, name: &str) -> Option<&'a Gc<Node>> {
    self.named.find(&name.to_str())
  }

  /// Returns a root node by path or None, if not found.
  pub fn get_by_path<'a>(&'a self, name: &str) -> Option<&'a Gc<Node>> {
    self.nodes.find(&name.to_str())
  }

  /// Returns true if PT has all of the requested root odes matched by path.
  /// Reports parser errors and returns false otherwise.
  pub fn expect_subnodes(&self, cx: &ExtCtxt, expectations: &[&str]) -> bool {
    let mut ok = true;
    for (path, sub) in self.nodes.iter() {
      if !expectations.contains(&path.as_slice()) {
        ok = false;
        cx.parse_sess().span_diagnostic.span_err(sub.path_span,
            format!("unknown root node `{}`", path).as_slice());
      }
    }
    ok
  }
}
