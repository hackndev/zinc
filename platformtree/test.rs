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

#![feature(phase)]
#![allow(unused_mut,dead_code)]

#[phase(plugin)] extern crate macro_platformtree;

use std::collections::hashmap;

mod pt;

#[test]
fn parse_basic_node() {
  let p = platformtree_parse!(
    node@root {}
  );

  assert!(p.name == Some("node".to_string()));
  assert!(p.path.path == vec!("root".to_string()));
  assert!(p.path.absolute == false);
}

#[test]
fn parse_path_string() {
  let p = platformtree_parse!(
    node@root::test::1 {}
  );

  assert!(p.path.path == vec!("root".to_string(), "test".to_string(), "1".to_string()));
}

#[test]
fn parse_absolute_path_string() {
  let p = platformtree_parse!(
    node@::root::test::1 {}
  );

  assert!(p.path.absolute == true);
}

#[test]
fn parse_anonymous_node() {
  let p = platformtree_parse!(
    @root {}
  );

  assert!(p.name == None);
}

#[test]
fn parse_node_attributes() {
  let p = platformtree_parse!(
    @root {
      str = "test";
      int = 10;
      ref = &test;
    }
  );

  assert!(p.attributes.get(&"str".to_string()) == &pt::StrValue("test".to_string()));
  assert!(p.attributes.get(&"int".to_string()) == &pt::UIntValue(10));
  assert!(p.attributes.get(&"ref".to_string()) == &pt::RefValue("test".to_string()));
}

#[test]
fn parse_child_nodes() {
  let p = platformtree_parse!(
    @root {
      sub@child {}
    }
  );

  assert!(p.subnodes.len() == 1);
  assert!(p.subnodes.get(0).name == Some("sub".to_string()));
}

#[test]
fn parse_anonymous_child_nodes() {
  let p = platformtree_parse!(
    @root {
      @child {}
    }
  );

  assert!(p.subnodes.len() == 1);
}
