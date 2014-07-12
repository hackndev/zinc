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

use std::rc::Rc;
use std::gc::Gc;
use syntax::ast;
use syntax::codemap::MacroBang;
use syntax::codemap::{CodeMap, Span, mk_sp, BytePos, ExpnInfo, NameAndSpan};
use syntax::codemap;
use syntax::diagnostic::{Emitter, RenderSpan, Level, mk_span_handler, mk_handler};
use syntax::ext::base::ExtCtxt;
use syntax::ext::expand::ExpansionConfig;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::parse::new_parse_sess_special_handler;
use syntax::print::pprust;

use builder::Builder;
use node;
use parser::Parser;

use hamcrest::{success,Matcher,MatchResult,SelfDescribing};
use std::fmt::Show;

pub struct EqualToString<T> {
  expected: T
}

impl<T: Show> SelfDescribing for EqualToString<T> {
  fn describe(&self) -> String {
    format!("{}", self.expected)
  }
}

impl<T : PartialEq+Show> Matcher<T> for EqualToString<T> {
  fn matches(&self, actual: T) -> MatchResult {
    if self.expected.eq(&actual) {
      success()
    }
    else {
      Err(format!("was {}", actual))
    }
  }
}

pub fn equal_to<T : PartialEq+Show>(expected: T) -> Box<EqualToString<T>> {
  box EqualToString { expected: expected }
}

pub fn equal_to_s(expected: &str) -> Box<EqualToString<String>> {
  equal_to(expected.to_string())
}

pub fn fails_to_parse(src: &str) {
  with_parsed_tts(src, |_, failed, pt| {
    assert!(unsafe{*failed} == true);
    assert!(pt.is_none());
  });
}

pub fn fails_to_build(src: &str) {
  with_parsed(src, |cx, failed, pt| {
    Builder::build(cx, pt);
    assert!(unsafe{*failed} == true);
  });
}

pub fn with_parsed(src: &str, block: |&mut ExtCtxt, *mut bool, Rc<node::PlatformTree>|) {
  with_parsed_tts(src, |cx, failed, pt| {
    assert!(unsafe{*failed} == false);
    block(cx, failed, pt.unwrap());
  });
}

pub fn with_parsed_node(name: &str, src: &str, block: |Rc<node::Node>|) {
  with_parsed(src, |_, _, pt| {
    block(pt.get_by_path(name).unwrap());
  });
}

pub fn with_parsed_tts(src: &str, block: |&mut ExtCtxt, *mut bool, Option<Rc<node::PlatformTree>>|) {
  let mut failed = false;
  let failptr = &mut failed as *mut bool;
  let ce = box CustomEmmiter::new(failptr);
  let sh = mk_span_handler(mk_handler(ce), CodeMap::new());
  let parse_sess = new_parse_sess_special_handler(sh);
  let cfg = Vec::new();
  let ecfg = ExpansionConfig {
    deriving_hash_type_parameter: false,
    crate_name: from_str("test").unwrap(),
  };
  let mut cx = ExtCtxt::new(&parse_sess, cfg, ecfg);
  cx.bt_push(ExpnInfo {
    call_site: mk_sp(BytePos(0), BytePos(0)),
    callee: NameAndSpan {
      name: "platformtree".to_string(),
      format: MacroBang,
      span: None,
    },
  });
  let tts = cx.parse_tts(src.to_string());

  let pt = Parser::new(&mut cx, tts.as_slice()).parse_platformtree();

  block(&mut cx, failptr, pt);
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
  fn emit(&mut self, _: Option<(&codemap::CodeMap, Span)>, m: &str, _: Option<&str>, l: Level) {
    unsafe { *self.failed = true };
    println!("{} {}", l, m);
  }
  fn custom_emit(&mut self, _: &codemap::CodeMap, _: RenderSpan, _: &str,
      _: Level) {
    fail!();
  }
}

pub fn assert_equal_source(stmt: &Gc<ast::Stmt>, src: &str) {
  let gen_src = pprust::stmt_to_string(stmt.deref());
  println!("generated: {}", gen_src);
  println!("expected:  {}", src);

  let stripped_gen_src = gen_src.replace(" ", "").replace("\n", "");
  let stripped_src = src.replace(" ", "").replace("\n", "");

  assert!(stripped_gen_src == stripped_src);
}

pub fn assert_equal_items(stmt: &Gc<ast::Item>, src: &str) {
  let gen_src = pprust::item_to_string(stmt.deref());
  println!("generated: {}", gen_src);
  println!("expected:  {}", src);

  let stripped_gen_src = gen_src.replace(" ", "").replace("\n", "");
  let stripped_src = src.replace(" ", "").replace("\n", "");

  assert!(stripped_gen_src == stripped_src);
}
