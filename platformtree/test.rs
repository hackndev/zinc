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
use syntax::parse::{new_parse_sess, new_parse_sess_special_handler};
use syntax::ext::expand::ExpansionConfig;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::diagnostic::{Emitter, RenderSpan, Level, mk_span_handler, mk_handler};
use syntax::codemap;
use syntax::codemap::{Span, CodeMap};

use parser::Parser;
use node;

#[test]
fn parse_basic_node() {
  with_parsed_node("node@root {}", |node: &node::Node| {
    assert!(node.name == Some("node".to_str()));
    assert!(node.path.path == vec!("root".to_str()));
    assert!(node.path.absolute == false);
  });
}

#[test]
fn parse_path_string() {
  with_parsed_node("node@root::test::1 {}", |node: &node::Node| {
    assert!(node.path.path == vec!("root".to_string(), "test".to_string(), "1".to_string()));
  });
}

#[test]
fn parse_absolute_path_string() {
  with_parsed_node("node@::root::test::1 {}", |node: &node::Node| {
    assert!(node.path.absolute == true);
  });
}

#[test]
fn parse_anonymous_node() {
  with_parsed_node("@root {}", |node: &node::Node| {
    assert!(node.name == None);
  });
}

#[test]
fn parse_node_attributes() {
  with_parsed_node("
    @root {
      str = \"test\";
      int = 10;
      ref = &test;
    }", |node: &node::Node| {
    assert!(node.attributes.get(&"str".to_string()) == &node::StrValue("test".to_string()));
    assert!(node.attributes.get(&"int".to_string()) == &node::UIntValue(10));
    assert!(node.attributes.get(&"ref".to_string()) == &node::RefValue("test".to_string()));
  });
}

#[test]
fn parse_child_nodes() {
  with_parsed_node("
    @root {
      sub@child {}
    }", |node: &node::Node| {
    assert!(node.subnodes.len() == 1);
    assert!(node.subnodes.get(0).name == Some("sub".to_string()));
  });
}

#[test]
fn parse_anonymous_child_nodes() {
  with_parsed_node("
    @root {
      @child {}
    }", |node: &node::Node| {
    assert!(node.subnodes.len() == 1);
  });
}

#[test]
fn doesnt_parse_empty_pt() {
  expect_failure("");
}

#[test]
fn doesnt_parse_node_with_no_body() {
  expect_failure("node@root");
}

#[test]
fn doesnt_parse_node_with_no_path() {
  expect_failure("node@ {}");
}

#[test]
fn doesnt_parse_node_with_broken_path() {
  expect_failure("node@::root::::blah {}");
}

#[test]
fn doesnt_parse_trailing_garbage() {
  expect_failure("node@root {} node@root {}");
}

fn expect_failure(src: &str) {
  let ce = CustomEmmiter::new();
  let sh = mk_span_handler(mk_handler(box ce), CodeMap::new());
  let parse_sess = new_parse_sess_special_handler(sh);
  let cfg = Vec::new();
  let ecfg = ExpansionConfig {
    deriving_hash_type_parameter: false,
    crate_id: from_str("test").unwrap(),
  };
  let cx = ExtCtxt::new(&parse_sess, cfg, ecfg);
  let tts = cx.parse_tts(src.to_str());

  let mut parser = Parser::new(&cx, tts.as_slice());
  let p = parser.parse_node();
  let failed_parse = match p {
    Ok(p) => false,
    Err(e) => true,
  };
  parser.should_finish();

  assert!(ce.failed() == true || failed_parse);
}

fn with_parsed_node(src: &str, block: |node: &node::Node|) {
  let parse_sess = new_parse_sess();
  let cfg = Vec::new();
  let ecfg = ExpansionConfig {
    deriving_hash_type_parameter: false,
    crate_id: from_str("test").unwrap(),
  };
  let cx = ExtCtxt::new(&parse_sess, cfg, ecfg);
  let tts = cx.parse_tts(src.to_str());

  let mut parser = Parser::new(&cx, tts.as_slice());
  let p = match parser.parse_node() {
    Ok(n) => n,
    Err(e) => {
      assert!(false);
      fail!();
    },
  };
  parser.should_finish();

  block(&p);
}

struct CustomEmmiter {
  failed: bool,
}

impl CustomEmmiter {
  pub fn new() -> CustomEmmiter {
    CustomEmmiter {
      failed: false,
    }
  }

  pub fn failed(&self) -> bool {
    self.failed
  }
}

impl Emitter for CustomEmmiter {
  fn emit(&mut self, cmsp: Option<(&codemap::CodeMap, Span)>,
          msg: &str, lvl: Level) {
    self.failed = true;
  }
  fn custom_emit(&mut self, cm: &codemap::CodeMap,
                 sp: RenderSpan, msg: &str, lvl: Level) {
    self.failed = true;
  }
}
