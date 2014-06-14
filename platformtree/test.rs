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
use syntax::parse::new_parse_sess;
use syntax::ext::expand::ExpansionConfig;
use syntax::ext::quote::rt::ExtParseUtils;

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
  let p = parser.parse_node();
  parser.should_finish();

  block(&p);
}
