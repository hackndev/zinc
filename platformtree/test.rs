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

use syntax::ext::base::ExtCtxt;
use syntax::parse::new_parse_sess_special_handler;
use syntax::ext::expand::ExpansionConfig;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::diagnostic::{Emitter, RenderSpan, Level, mk_span_handler, mk_handler};
use syntax::codemap;
use syntax::codemap::{Span, CodeMap};
use std::gc::Gc;

use parser::Parser;
use node;

#[test]
fn parse_anonymous_node() {
  with_parsed_node("root {}", |node| {
    assert!(node.name == None);
    assert!(node.path == "root".to_str());
    assert!(node.attributes.len() == 0);
    assert!(node.subnodes.len() == 0);
  });
}

#[test]
fn parse_node_with_name() {
  with_parsed_node("test@root {}", |node| {
    assert!(node.name == Some("test".to_str()));
    assert!(node.path == "root".to_str());
    assert!(node.attributes.len() == 0);
    assert!(node.subnodes.len() == 0);
  });
}

#[test]
fn fails_to_parse_node_with_bad_name() {
  fails_to_parse("1@root {}");
  fails_to_parse("@root {}");
  fails_to_parse("+@root {}");
}

#[test]
fn fails_to_parse_node_with_bad_path() {
  fails_to_parse("test root {}");
  fails_to_parse("test@ {}");
  fails_to_parse("test@- {}");
}

#[test]
fn parse_node_with_no_body() {
  with_parsed_node("test@root;", |_| {
    assert!(true);
  });
}

#[test]
fn parse_node_with_numeric_path() {
  with_parsed_node("test@1 {}", |node| {
    assert!(node.path == "1".to_str());
  });
}

#[test]
fn parse_attributes() {
  with_parsed_node("test@root { a = \"value\"; b = 1; c = &ref; }", |node| {
    assert!(node.get_string_attr("a") == Some(&"value".to_str()));
    assert!(node.get_int_attr("b")    == Some(1));
    assert!(node.get_ref_attr("c")    == Some(&"ref".to_str()));
  });
}

#[test]
fn parse_string_attribute() {
  with_parsed_node("test@root { key = \"value\"; }", |node| {
    assert!(node.get_string_attr("key") == Some(&"value".to_str()));
  });
}

#[test]
fn parse_integer_attribute() {
  with_parsed_node("test@root { key = 10; }", |node| {
    assert!(node.get_int_attr("key") == Some(10));
  });
}

#[test]
fn parse_ref_attribute() {
  with_parsed_node("test@root { key = &ref; }", |node| {
    assert!(node.get_ref_attr("key") == Some(&"ref".to_str()));
  });
}

#[test]
fn fails_to_parse_duplicate_attributes() {
  fails_to_parse("test@root { a = 1; a = \"2\"; }");
}

#[test]
fn fails_to_parse_malformed_attibute() {
  fails_to_parse("test@root { k = \"value\" }");
  fails_to_parse("test@root { 1 = \"value\"; }");
  fails_to_parse("test@root { k = v; }");
  fails_to_parse("test@root { k = 10u8; }");
  fails_to_parse("test@root { k = 10i8; }");
  fails_to_parse("test@root { k = -42; }");
  fails_to_parse("test@root { k = &1; }");
  fails_to_parse("test@root { k = &\"q\"; }");
}

#[test]
fn parse_anonymous_subnode() {
  with_parsed_node("test@root { child; }", |node| {
    assert!(node.subnodes.len() == 1);
  });
  with_parsed_node("test@root { child {} }", |node| {
    assert!(node.subnodes.len() == 1);
  });
}

#[test]
fn parse_named_subnode() {
  with_parsed_node("test@root { sub@child; }", |node| {
    assert!(node.subnodes.len() == 1);
  });
  with_parsed_node("test@root { sub@child {} }", |node| {
    assert!(node.subnodes.len() == 1);
  });
}

// helpers
fn fails_to_parse(src: &str) {
  with_parsed_tts(src, |failed, pt| {
    assert!(failed == true);
    assert!(pt.is_none());
  });
}

fn with_parsed(src: &str, block: |node: &node::PlatformTree|) {
  with_parsed_tts(src, |failed, pt| {
    assert!(failed == false);
    block(&pt.unwrap());
  });
}

fn with_parsed_node(src: &str, block: |node: &Gc<node::Node>|) {
  with_parsed(src, |pt| {
    assert!(pt.nodes.len() == 1);
    block(pt.nodes.get(0));
  });
}

fn with_parsed_tts(src: &str, block: |bool, Option<node::PlatformTree>|) {
  let mut failed = false;
  let failptr = &mut failed as *mut bool;
  let ce = box CustomEmmiter::new(failptr);
  let sh = mk_span_handler(mk_handler(ce), CodeMap::new());
  let parse_sess = new_parse_sess_special_handler(sh);
  let cfg = Vec::new();
  let ecfg = ExpansionConfig {
    deriving_hash_type_parameter: false,
    crate_id: from_str("test").unwrap(),
  };
  let cx = ExtCtxt::new(&parse_sess, cfg, ecfg);
  let tts = cx.parse_tts(src.to_str());

  let mut parser = Parser::new(&cx, tts.as_slice());
  let nodes = parser.parse_platformtree();

  block(failed, nodes);
}

struct CustomEmmiter {
  failed: *mut bool
}

impl CustomEmmiter {
  pub fn new(fp: *mut bool) -> CustomEmmiter {
    CustomEmmiter {
      failed: fp,
    }
  }
}

impl Emitter for CustomEmmiter {
  fn emit(&mut self, _: Option<(&codemap::CodeMap, Span)>, m: &str, l: Level) {
    unsafe { *self.failed = true };
    println!("{} {}", l, m);
  }
  fn custom_emit(&mut self, _: &codemap::CodeMap, _: RenderSpan, _: &str,
      _: Level) {
    fail!();
  }
}
