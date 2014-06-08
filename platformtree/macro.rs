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

#![crate_id="platformtree_macro"]
#![crate_type="dylib"]

#![feature(macro_registrar, quote, managed_boxes)]

extern crate syntax;

use syntax::ast::{Name, TokenTree};
use syntax::ast;
use syntax::codemap::{Span};
use syntax::ext::base::{SyntaxExtension, BasicMacroExpander, NormalTT, ExtCtxt};
use syntax::ext::base::{MacResult, MacExpr};
use syntax::ext::base;
use syntax::parse::{token, ParseSess, lexer};
use syntax::print::pprust::expr_to_str;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};

use std::collections::hashmap;

mod pt;

#[macro_registrar]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
  register(token::intern("platformtree_parse"), NormalTT(
    box BasicMacroExpander {
      expander: platformtree_parse,
      span: None,
    },
    None)
  );
}

/// Parse paltformtree to pt::Node
pub fn platformtree_parse(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
    -> Box<MacResult> {
  let mut parser = Parser::new(cx, tts);

  let node = parser.parse_node();

  println!("PT parsed: {}", node);

  base::MacExpr::new(quote_expr!(&*cx, $node))
}

impl pt::Node {
  pub fn to_source(&self) -> String {
    let mappath: Vec<String> = self.path.path.iter().map(|x| format!("\"{}\".to_string()", x)).collect();
    let node_struct = format!("pt::Node \\{ \
        name: {}, \
        path: pt::Path \\{ absolute: {}, path: vec!({}) \\}, \
        attributes: hashmap::HashMap::new(), \
        subnodes: Vec::new() \
      \\}",
      match self.name {
        Some(ref s) => format!("Some(\"{}\".to_string())", s),
        None => "None".to_string(),
      },
      self.path.absolute,
      mappath.connect(", "));

    let mut init_chunks = "".to_string();
    for (k, v) in self.attributes.iter() {
      let strinified_val = match v {
        &pt::UIntValue(ref u) => format!("pt::UIntValue({})", u),
        &pt::StrValue(ref s)  => format!("pt::StrValue(\"{}\".to_string())", s),
        &pt::RefValue(ref r)  => format!("pt::RefValue(\"{}\".to_string())", r),
      };

      init_chunks = init_chunks.append(format!("attrs.insert(\"{}\".to_string(), {});",
          k, strinified_val).as_slice());
    }

    for sn in self.subnodes.iter() {
      init_chunks = init_chunks.append(format!(
          "nodes.push(box {});", sn.to_source()).as_slice());
    }

    // TODO(farcaller): this triggers unused_mut for some reason.
    let init_struct = format!("
      \\{
        let mut node = {};
        let mut attrs = hashmap::HashMap::new();
        let mut nodes = Vec::new();
        {}
        node.attributes = attrs;
        node.subnodes = nodes;
        node
      \\}", node_struct, init_chunks);

    init_struct
  }
}

impl ToTokens for pt::Node {
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
    (cx as &ExtParseUtils).parse_tts(self.to_source())
  }
}

struct Parser<'a> {
  pub sess: &'a ParseSess,
  reader: Box<lexer::Reader:>,

  token: token::Token,
  span: Span,

  last_token: Option<Box<token::Token>>,
  last_span: Span,

  backlog: Vec<lexer::TokenAndSpan>,
}

impl<'a> Parser<'a> {
  pub fn new<'a>(cx: &'a ExtCtxt, tts: &[TokenTree]) -> Parser<'a> {
    let sess = cx.parse_sess();
    let ttsvec = tts.iter().map(|x| (*x).clone()).collect();
    let mut reader = box lexer::new_tt_reader(&sess.span_diagnostic, None, ttsvec)
        as Box<lexer::Reader>;

    let tok0 = reader.next_token();
    let token = tok0.tok;
    let span = tok0.sp;

    Parser {
      sess: sess,
      reader: reader,

      token: token,
      span: span,

      last_token: None,
      last_span: span,

      backlog: Vec::new(),
    }
  }

  fn fatal(&self, m: &str) -> ! {
    self.sess.span_diagnostic.span_fatal(self.span, m);
  }

  fn bump(&mut self) -> token::Token {
    let tok = self.token.clone();
    self.last_span = self.span;
    self.last_token = Some(box tok.clone());

    let next = match self.backlog.pop() {
      Some(ts) => ts,
      None => self.reader.next_token(),
    };

    self.span = next.sp;
    self.token = next.tok;

    tok
  }

  fn unbump(&mut self) {
    let span = self.last_span;
    let boxtok: &Box<token::Token> = match self.last_token {
      Some(ref t) => t,
      None => fail!(),
    };
    let tok = *boxtok.clone();

    self.backlog.push(lexer::TokenAndSpan {
      tok: self.token.clone(),
      sp: self.span,
    });

    self.token = tok;
    self.span = span;
  }

  fn expect(&mut self, t: &token::Token) {
    if self.token == *t {
      self.bump();
    } else {
      let token_str = token::to_str(t);
      let this_token_str = token::to_str(&self.token);
      self.fatal(format!("expected `{}` but found `{}`", token_str, this_token_str).as_slice())
    }
  }

  fn expect_ident(&mut self) -> token::Token {
    match self.token {
      token::IDENT(_, _) => {
        self.bump()
      },
      _ => {
        let this_token_str = token::to_str(&self.token);
        self.fatal(format!("expected identifier but found `{}`", this_token_str).as_slice())
      },
    }
  }

  pub fn parse_node(&mut self) -> pt::Node {
    let mut node = pt::Node::new();
    // NODE_ID @ NODE_PATH { CONTENTS }
    // or
    //         @ NODE_PATH { CONTENTS }

    node.name = match self.token {
      token::IDENT(_, _) => {
        let ret = Some(token::to_str(&self.token));
        self.bump();
        ret
      },
      token::AT => {
        None
      },
      _ => {
        let this_token_str = token::to_str(&self.token);
        self.fatal(format!("expected identifier or `@` but found `{}`", this_token_str).as_slice())
      },
    };

    self.expect(&token::AT);

    node.path = self.parse_path();

    self.expect(&token::LBRACE);
    self.parse_node_contents(&mut node);
    self.expect(&token::RBRACE);

    node
  }

  pub fn parse_node_contents(&mut self, node: &mut pt::Node) {
    let mut attrs = hashmap::HashMap::new();
    let mut nodes = Vec::new();

    loop {
      match self.token {
        token::RBRACE => break,
        token::IDENT(_, _) => {
          let name = token::to_str(&self.token);
          self.bump();

          match self.token {
            token::EQ => {
              self.bump();
              let val = self.parse_attribute_value();
              self.expect(&token::SEMI);

              attrs.insert(name, val);
            },
            token::AT => {
              self.unbump();
              let node = self.parse_node();
              nodes.push(box node);
            },
            _ => {
              let this_token_str = token::to_str(&self.token);
              self.fatal(format!("expected `@` or `=` but found `{}`", this_token_str).as_slice())
            }
          };
        }
        _ => {
          let this_token_str = token::to_str(&self.token);
          self.fatal(format!("expected identifier or `\\}` but found `{}`", this_token_str).as_slice())
        }
      }
    }

    node.attributes = attrs;
    node.subnodes = nodes;
  }

  pub fn parse_attribute_value(&mut self) -> pt::AttributeValue {
    let val = match self.token {
      token::LIT_STR(sv) => {
        let val = pt::StrValue(token::get_ident(sv).get().to_string());
        self.bump();
        val
      },
      token::LIT_INT_UNSUFFIXED(intval) => {
        let val = pt::UIntValue(intval as uint);
        self.bump();
        val
      },
      token::BINOP(token::AND) => {
        self.bump();
        pt::RefValue(token::to_str(&self.expect_ident()).to_string())
      },
      _ => {
        let this_token_str = token::to_str(&self.token);
        self.fatal(format!("expected integer or string but found {} `{}`", self.token, this_token_str).as_slice())
      }
    };

    val
  }

  pub fn parse_path(&mut self) -> pt::Path {
    let mut v = Vec::new();
    let mut expect_more = false;
    let mut absolute = false;

    if self.token == token::MOD_SEP {
      self.bump();
      expect_more = true;
      absolute = true;
    }

    loop {
      match self.token {
        token::MOD_SEP => {
          if expect_more {
            let this_token_str = token::to_str(&self.token);
            self.fatal(format!("expected identifier but found `{}`", this_token_str).as_slice())
          }
          self.bump();
          expect_more = true;
        },
        token::IDENT(_, _) => {
          v.push(token::to_str(&self.token));
          self.bump();
          expect_more = false;
        },
        token::LIT_INT_UNSUFFIXED(u) => {
          v.push(u.to_str());
          self.bump();
          expect_more = false;
        }
        _ => {
          if !expect_more {
            if v.len() == 0 {
              self.fatal(format!("unfinished path, found {} `{}`", self.token,
                  token::to_str(&self.token)).as_slice());
            }
            break
          } else {
            self.fatal(format!("unfinished path, found {} `{}`", self.token,
                token::to_str(&self.token)).as_slice());
          }
        }
      }
    }
    pt::Path {
      absolute: absolute,
      path: v,
    }
  }
}
