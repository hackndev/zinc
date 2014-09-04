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

use test_helpers::{fails_to_parse, with_parsed, with_parsed_node};

#[test]
fn parse_anonymous_node() {
  with_parsed_node("node", "node {}", |node| {
    assert!(node.name == None);
    assert!(node.path == "node".to_string());
    assert!(node.attributes.borrow().len() == 0);
    assert!(node.subnodes().len() == 0);
  });
}

#[test]
fn parse_node_with_name() {
  with_parsed_node("root", "test@root {}", |node| {
    assert!(node.name == Some("test".to_string()));
    assert!(node.path == "root".to_string());
    assert!(node.attributes.borrow().len() == 0);
    assert!(node.subnodes().len() == 0);
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
fn fails_to_parse_node_with_repeated_paths() {
  fails_to_parse("root@node { sub; sub; }");
}

#[test]
fn fails_to_parse_root_nodes_with_repeated_paths() {
  fails_to_parse("root@node; root2@node;");
}

#[test]
fn parse_node_with_no_body() {
  with_parsed_node("root", "test@root;", |_| {
    assert!(true);
  });
}

#[test]
fn parse_node_with_numeric_path() {
  with_parsed_node("1", "test@1 {}", |node| {
    assert!(node.path == "1".to_string());
  });
}

#[test]
fn parse_attributes() {
  with_parsed_node("root", "test@root { a = \"value\"; b = 1; c = &ref; }", |node| {
    assert!(node.get_string_attr("a") == Some("value".to_string()));
    assert!(node.get_int_attr("b")    == Some(1));
    assert!(node.get_ref_attr("c")    == Some("ref".to_string()));
  });
}

#[test]
fn parse_string_attribute() {
  with_parsed_node("root", "test@root { key = \"value\"; }", |node| {
    assert!(node.get_string_attr("key") == Some("value".to_string()));
  });
}

#[test]
fn parse_integer_attribute() {
  with_parsed_node("root", "test@root { key = 10; }", |node| {
    assert!(node.get_int_attr("key") == Some(10));
  });
}

#[test]
fn parse_ref_attribute() {
  with_parsed_node("root", "test@root { key = &ref; }", |node| {
    assert!(node.get_ref_attr("key") == Some("ref".to_string()));
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
  with_parsed_node("root", "test@root { child; }", |node| {
    assert!(node.subnodes().len() == 1);
  });
  with_parsed_node("root", "test@root { child {} }", |node| {
    assert!(node.subnodes().len() == 1);
  });
}

#[test]
fn parse_named_subnode() {
  with_parsed_node("root", "test@root { sub@child; }", |node| {
    assert!(node.subnodes().len() == 1);
  });
  with_parsed_node("root", "test@root { sub@child {} }", |node| {
    assert!(node.subnodes().len() == 1);
  });
}

#[test]
fn tracks_nodes_by_name() {
  with_parsed("test@root { sub@child; }", |_, _, pt| {
    let subnode = pt.get_by_name("sub");
    assert!(subnode.is_some());
    assert!(subnode.unwrap().path == "child".to_string());
  });
}

#[test]
fn fails_to_parse_duplicate_node_names() {
  fails_to_parse("duplicate@root { duplicate@child; }");
}
