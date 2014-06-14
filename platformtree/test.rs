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
fn parse_node_with_no_name() {
  with_parsed_node("root {}", |node: &Gc<node::Node>| {
    assert!(node.name == None);
    assert!(node.path == "root".to_str());
    assert!(node.attributes.len() == 0);
    assert!(node.subnodes.len() == 0);
  });
}

fn with_parsed_node(src: &str, block: |node: &Gc<node::Node>|) {
  with_parsed(src, |pt: &node::PlatformTree| {
    assert!(pt.nodes.len() == 1);
    block(pt.nodes.get(0));
  });
}

fn with_parsed(src: &str, block: |node: &node::PlatformTree|) {
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
  let nodes = parser.parse_platformtree().unwrap();
  assert!(ce.failed() == false);

  block(&nodes);
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
  fn emit(&mut self, _: Option<(&codemap::CodeMap, Span)>, _: &str, _: Level) {
    self.failed = true;
  }
  fn custom_emit(&mut self, _: &codemap::CodeMap, _: RenderSpan, _: &str,
      _: Level) {
    self.failed = true;
  }
}
