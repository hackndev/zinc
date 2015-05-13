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
use std::collections::HashMap;
use std::fmt;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;

use builder::Builder;

pub use self::AttributeValue::*;
pub use self::AttributeType::*;

/// Holds a value for an attribute.
///
/// The value can be an unsigned integer, string or reference.
#[derive(Clone)]
pub enum AttributeValue {
  IntValue(usize),
  BoolValue(bool),
  StrValue(String),
  RefValue(String),
}

/// Expected attribute type.
///
/// Used in Node::expect_attributes to provide the expected type of the
/// attribute.
#[derive(Copy)]
pub enum AttributeType {
  IntAttribute,
  BoolAttribute,
  StrAttribute,
  RefAttribute,
}

/// Attribute value and metadata.
///
/// Stored inside of a HashMap, the key to HashMap is the attribute name.
/// Provides spans for both key and value.
#[derive(Clone)]
pub struct Attribute {
  pub value: AttributeValue,
  pub key_span: Span,
  pub value_span: Span,
}

impl Attribute {
  /// Creates a new attribute with given span.
  pub fn new(value: AttributeValue, key_span: Span, value_span: Span)
      -> Attribute {
    Attribute {
      value: value,
      key_span: key_span,
      value_span: value_span,
    }
  }

  /// Creates a new attribute with DUMMY_SP for key and value.
  pub fn new_nosp(value: AttributeValue) -> Attribute {
    Attribute {
      value: value,
      key_span: DUMMY_SP,
      value_span: DUMMY_SP,
    }
  }
}

/// Node builder is a function that generates code based on the node content or
/// mutates other nodes.
pub type NodeBuilderFn = fn(&mut Builder, &mut ExtCtxt, Rc<Node>);

/// Subnodes is effectively an ordered map.
///
/// We still address nodes by path for most of use cases, but we need to know
/// the original order of appearance (it makes things deterministic).
pub struct Subnodes {
  by_index: Vec<Rc<Node>>,
  by_path: HashMap<String, Weak<Node>>,
}

impl Subnodes {
  pub fn new() -> Subnodes {
    Subnodes {
      by_index: vec!(),
      by_path: HashMap::new(),
    }
  }

  /// Adds a node to subnodes.
  ///
  /// The node must not be present in the subnodes.
  pub fn push(&mut self, node: Rc<Node>) {
    let weak = node.downgrade();
    self.by_path.insert(node.path.clone(), weak);
    self.by_index.push(node);
  }

  /// Returns a vector representation of subnodes.
  pub fn as_vec<'a>(&'a self) -> &'a Vec<Rc<Node>> {
    &self.by_index
  }

  /// Returns a map representation of subnodes.
  pub fn as_map<'a>(&'a self) -> &'a HashMap<String, Weak<Node>> {
    &self.by_path
  }

  /// A helper method to move data from other subnodes into current instance.
  ///
  /// Used as a helper for wrapping in RefCell.
  pub fn clone_from(&mut self, other: Subnodes) {
    self.by_index = other.by_index;
    self.by_path = other.by_path;
  }
}

/// PlatformTree node.
///
/// Might have an optional name, is the name is missing, name_span is equal to
/// path_span. Attributes are stored by name, subnodes are stored by path.
/// Type_name, if present, must specify the type path for the node's
/// materialized object.
///
/// Two nodes are equal if their full paths are equal.
pub struct Node {
  /// Node name, might be optional.
  pub name: Option<String>,

  /// Name span if name is present or path span otherwise.
  pub name_span: Span,

  /// Node path, which is unique among all sibling nodes.
  pub path: String,

  /// Path span.
  pub path_span: Span,

  /// A map of node's attributes.
  pub attributes: RefCell<HashMap<String, Rc<Attribute>>>,

  /// A weak reference to parent node, None for root nodes.
  pub parent: Option<Weak<Node>>,

  /// A function that materializes this node, i.e. generates some actionable
  /// code.
  ///
  /// Materializers are exectuted in order of dependencies resolution, so having
  /// a fully built tree of dependencies is a must.
  pub materializer: Cell<Option<NodeBuilderFn>>,

  /// Present iff this node will modify state of any other nodes.
  ///
  /// Mutators are executed before materializers in no specific order.
  pub mutator: Cell<Option<NodeBuilderFn>>,

  /// List of nodes that must be materialized before this node.
  ///
  /// Generally, a node must depend on something to be materialized. The root
  /// node that all other nodes must depend on implicitly or explicitly is
  /// mcu::clock, which must always be present in PT.
  pub depends_on: RefCell<Vec<Weak<Node>>>,

  /// List of nodes that may be materialized before this node.
  pub rev_depends_on: RefCell<Vec<Weak<Node>>>,

  subnodes: RefCell<Subnodes>,
  type_name: RefCell<Option<String>>,
  type_params: RefCell<Vec<String>>,
}

impl Node {
  pub fn new(name: Option<String>, name_span: Span, path: String,
      path_span: Span, parent: Option<Weak<Node>>) -> Node {
    Node {
      name: name,
      name_span: name_span,
      path: path,
      path_span: path_span,
      attributes: RefCell::new(HashMap::new()),
      subnodes: RefCell::new(Subnodes::new()),
      parent: parent,
      type_name: RefCell::new(None),
      type_params: RefCell::new(vec!()),
      materializer: Cell::new(None),
      mutator: Cell::new(None),
      depends_on: RefCell::new(Vec::new()),
      rev_depends_on: RefCell::new(Vec::new()),
    }
  }

  /// Set type name for the generated struct.
  ///
  /// If this node generates an object in main(), type_name references the type
  /// of that object, e.g. for DHT22 driver that would be
  /// `zinc::drivers::dht22::DHT22`.
  pub fn set_type_name(&self, tn: String) {
    let mut borrow = self.type_name.borrow_mut();
    borrow.deref_mut().clone_from(&Some(tn));
  }

  /// Get type name for the generated struct.
  pub fn type_name(&self) -> Option<String> {
    self.type_name.borrow().clone()
  }

  /// Get type parameters for the generated object.
  pub fn type_params(&self) -> Vec<String> {
    self.type_params.borrow().clone()
  }

  /// Set type parameters for the generated object, including lifetimes.
  ///
  /// A default lifetime if this is going to end as a task argument is 'a. Other
  /// lifetimes or fully-qualified types may be used as well. DHT22 driver uses
  /// this to provide `zinc::hal::timer::Timer` and `zinc::hal::pin::GPIO` for
  /// its public struct of `pub struct DHT22<'a, T, P>`.
  pub fn set_type_params(&self, params: Vec<String>) {
    let mut borrow = self.type_params.borrow_mut();
    borrow.deref_mut().clone_from(&params);
  }

  /// Returns a cloned vec of node's subnodes.
  pub fn subnodes(&self) -> Vec<Rc<Node>> {
    self.subnodes.borrow().as_vec().clone()
  }

  /// Invokes the closure for each node from node's subnodes passing a path and
  /// weak node reference.
  pub fn with_subnodes_map<F>(&self, mut f: F)
      where F: FnMut(&HashMap<String, Weak<Node>>) {
    let borrow = self.subnodes.borrow();
    f(borrow.as_map());
  }

  /// Sets the node's subnodes from a passed object.
  pub fn set_subnodes(&self, new: Subnodes) {
    self.subnodes.borrow_mut().clone_from(new);
  }

  /// Returns a clones string of current node's path.
  pub fn path(&self) -> String {
    self.path.clone()
  }

  /// Returns a fully-qualified path of the current node.
  pub fn full_path(&self) -> String {
    let pp = match self.parent {
      Some(ref parent) => parent.clone().upgrade().unwrap().full_path() + "::",
      None => "".to_string(),
    };
    pp + self.path.as_slice()
  }

  /// Returns attribute by name or panic!()s.
  pub fn get_attr(&self, key: &str) -> Rc<Attribute> {
    let ref attr = (*self.attributes.borrow())[key.to_string()];
    attr.clone()
  }

  /// Returns a string attribute by name or None, if it's not present or not of
  /// a StrAttribute type.
  pub fn get_string_attr(&self, key: &str) -> Option<String> {
    self.attributes.borrow().get(&key.to_string()).and_then(|av| match av.value {
      StrValue(ref s) => Some(s.clone()),
      _ => None,
    })
  }

  /// Returns an integer attribute by name or None, if it's not present or not
  /// of an IntAttribute type.
  pub fn get_int_attr(&self, key: &str) -> Option<usize> {
    self.attributes.borrow().get(&key.to_string()).and_then(|av| match av.value {
      IntValue(ref u) => Some(*u),
      _ => None,
    })
  }

  /// Returns a bool attribute by name or None, if it's not present or not
  /// of an BoolAttribute type.
  pub fn get_bool_attr(&self, key: &str) -> Option<bool> {
    self.attributes.borrow().get(&key.to_string()).and_then(|av| match av.value {
      BoolValue(ref b) => Some(*b),
      _ => None,
    })
  }

  /// Returns a reference attribute by name or None, if it's not present or not
  /// of a RefAttribute type.
  pub fn get_ref_attr(&self, key: &str) -> Option<String> {
    self.attributes.borrow().get(&key.to_string()).and_then(|av| match av.value {
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
      -> Option<usize> {
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

  /// Returns a boolean attribute by name or None if it's not present or not
  /// of an BoolAttribute type. Reports a parser error if an attribute is
  /// missing.
  pub fn get_required_bool_attr(&self, cx: &ExtCtxt, key: &str)
      -> Option<bool> {
    match self.get_bool_attr(key) {
      Some(val) => Some(val),
      None => {
        cx.parse_sess().span_diagnostic.span_err(self.name_span,
            format!("required boolean attribute `{}` is missing", key)
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
    for sub in self.subnodes().iter() {
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
        &BoolAttribute => {
          if self.get_required_bool_attr(cx, n).is_none() {ok = false}
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
    for sub in self.subnodes().iter() {
      if !expectations.contains(&sub.path.as_slice()) {
        ok = false;
        cx.parse_sess().span_diagnostic.span_err(sub.path_span,
            format!("unknown subnode `{}` in node `{}`",
                sub.path, self.path).as_slice());
      }
    }
    ok
  }

  /// Returns a subnode by path or None, if not found.
  pub fn get_by_path(&self, path: &str) -> Option<Rc<Node>> {
    self.subnodes.borrow().as_map().get(&path.to_string()).and_then(|node| {
      Some(node.clone().upgrade().unwrap())
    })
  }
}

impl PartialEq for Node {
  fn eq(&self, other: &Node) -> bool {
    self.full_path() == other.full_path()
  }

  fn ne(&self, other: &Node) -> bool {
    self.full_path() != other.full_path()
  }
}

impl fmt::Display for Node {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    fmt.write_str(format!("<Node {}>", self.full_path()).as_slice())
        .or_else(|_| { panic!() })
  }
}

/// PlatformTree root object.
///
/// Root nodes are stored by path in `nodes`, All the nmaed nodes are also
/// stored by name in `named`.
///
/// TODO(farcaller): this could be really refactored into transient root node
/// object that can depend on mcu::clock.
pub struct PlatformTree {
  nodes: HashMap<String, Rc<Node>>,
  named: HashMap<String, Weak<Node>>,
}

impl PlatformTree {
  pub fn new(nodes: HashMap<String, Rc<Node>>,
      named: HashMap<String, Weak<Node>>) -> PlatformTree {
    PlatformTree {
      nodes: nodes,
      named: named,
    }
  }

  pub fn nodes(&self) -> Vec<Rc<Node>> {
    let mut v = vec!();
    for (_, sub) in self.nodes.iter() {
      v.push(sub.clone())
    }
    v
  }

  /// Returns a node by name or None, if not found.
  pub fn get_by_name(&self, name: &str) -> Option<Rc<Node>> {
    self.named.get(&name.to_string()).and_then(|node| { Some(node.upgrade().unwrap().clone()) })
  }

  /// Returns a root node by path or None, if not found.
  pub fn get_by_path(&self, name: &str) -> Option<Rc<Node>> {
    self.nodes.get(&name.to_string()).and_then(|node| { Some(node.clone()) })
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
