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

#[allow(unused_imports)] use syntax::ast::TokenTree;
#[allow(unused_imports)] use syntax::codemap::{Span, mk_sp, BytePos};
#[allow(unused_imports)] use syntax::ext::base::ExtCtxt;
#[allow(unused_imports)] use syntax::parse::{token, ParseSess, lexer};
#[allow(unused_imports)] use syntax::ext::build::AstBuilder;
#[allow(unused_imports)] use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};

#[allow(unused_imports)] use std::collections::hashmap;

use node;
use std::gc::Gc;

#[allow(dead_code)]
pub struct Parser<'a> {
  pub sess: &'a ParseSess,
  reader: Box<lexer::Reader:>,
  token: token::Token,
  span: Span,

  last_token: Option<Box<token::Token>>,
  last_span: Span,
}

impl<'a> Parser<'a> {
  #[allow(dead_code)]
  pub fn new<'a>(cx: &'a ExtCtxt, tts: &[TokenTree]) -> Parser<'a> {
    let sess = cx.parse_sess();
    let ttsvec = tts.iter().map(|x| (*x).clone()).collect();
    let mut reader = box lexer::new_tt_reader(
        &sess.span_diagnostic, None, ttsvec) as Box<lexer::Reader>;

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
    }
  }

  /// Parse the platform tree from passed in tokens.
  pub fn parse_platformtree(&mut self) -> Option<node::PlatformTree> {
    let mut nodes: Vec<Gc<node::Node>> = Vec::new();
    let mut failed = false;
    loop {
      if self.token == token::EOF {
        break
      }

      let node = match self.parse_node() {
        Some(node) => box(GC) node,
        None => {
          failed = true;
          self.bump();
          continue
        }
      };

      nodes.push(node);
    }

    if failed {
      None
    } else {
      Some(node::PlatformTree::new(nodes))
    }
  }

  fn parse_node(&mut self) -> Option<node::Node> {
    let mut node_span: Span;
    let node_path_span: Span;
    let node_name: Option<String>;
    let attributes: hashmap::HashMap<String, node::Attribute>;

    // check if next token is @
    //    we're here
    //   /
    //   v
    //   NAME @ PATH { ... }
    //        ^-- peeking here
    if self.reader.peek().tok == token::AT {
      node_span = self.span;
      node_name = match self.expect_ident() {
        Some(name) => Some(name),
        None => return None,
      };
      if !self.expect(&token::AT) {
        return None;
      }
    } else {
      node_name = None;
      node_span = mk_sp(BytePos(0), BytePos(0));
    }

    // NAME is resolved, if it was there anyway.
    //    we're here
    //          |
    //          v
    //   NAME @ PATH { ... }
    node_path_span = self.span;
    if node_name == None {
      node_span = self.span;
    } else {
      node_span.hi = self.span.hi;
    }

    let node_path = match self.token {
      token::IDENT(_, _) => {
        token::to_str(&self.bump())
      },
      token::LIT_INT_UNSUFFIXED(u) => {
        self.bump();
        u.to_str()
      }
      ref other => {
        self.error(format!("expected node path but found `{}`",
            token::to_str(other)));
        return None;
      }
    };

    //    we're here
    //             |
    //             v
    // NAME @ PATH { ... }
    // it's either a body, or a semicolon (no body)
    match self.bump() {
      token::LBRACE => {
        attributes = match self.parse_node_body() {
          Some(attrs) => attrs,
          // TODO(farcaller): eat everything up to '}' and continue if failed
          // we can still parse further nodes.
          None => return None,
        };

        if !self.expect(&token::RBRACE) {
          return None;
        }
      },
      token::SEMI => {
        attributes = hashmap::HashMap::new();
      },
      ref other => {
        self.error(format!("expected `\\{` or `;` but found `{}`",
            token::to_str(other)));
        return None;
      }
    }

    let mut node = node::Node::new(node_name, node_span, node_path, node_path_span);
    node.attributes = attributes;
    Some(node)
  }

  fn parse_node_body(&mut self)
      -> Option<hashmap::HashMap<String, node::Attribute>> {
    let mut attrs = hashmap::HashMap::new();
    loop {
      // break early if at brace
      if self.token == token::RBRACE {
        break;
      }

      // we're here
      // |
      // v
      // ATTR = VAL ;
      let attr_key_span = self.span;
      let attr_name = match self.expect_ident() {
        Some(name) => name,
        None => return None,
      };

      if attrs.contains_key(&attr_name) {
        self.error(format!("key `{}` is already defined", attr_name));
        return None;
      }

      // we're here
      //      |
      //      v
      // ATTR = VAL ;
      if !self.expect(&token::EQ) {
        return None;
      }

      // we're here
      //        |
      //        v
      // ATTR = VAL ;
      let attr_value_span = self.span;
      let attr_value = match self.parse_attribute_value() {
        Some(value) => value,
        None => return None,
      };

      //   we're here
      //            |
      //            v
      // ATTR = VAL ;
      if !self.expect(&token::SEMI) {
        return None;
      }

      attrs.insert(attr_name, node::Attribute::new(
          attr_value, attr_key_span, attr_value_span));
    }

    Some(attrs)
  }

  fn parse_attribute_value(&mut self) -> Option<node::AttributeValue> {
    match self.token {
      token::LIT_STR(string_val) => {
        self.bump();
        let string = token::get_ident(string_val).get().to_str();
        Some(node::StrValue(string))
      },
      // TODO(farcaller): any other integers can surface here?
      token::LIT_INT_UNSUFFIXED(intval) => {
        self.bump();
        Some(node::UIntValue(intval as uint))
      },
      ref other => {
        self.error(format!("expected string but found `{}`",
            token::to_str(other)));
        None
      }
    }
  }

  fn error(&self, m: String) {
    self.sess.span_diagnostic.span_err(self.span, m.as_slice());
  }

  /// Bumps a token.
  ///
  /// This moves current token to last token, pops a new token from backlog or
  /// reader and returns the last token (i.e. the 'current' token at the time of
  /// method call).
  fn bump(&mut self) -> token::Token {
    let tok = self.token.clone();
    self.last_span = self.span;
    self.last_token = Some(box tok.clone());

    let next = self.reader.next_token();

    self.span = next.sp;
    self.token = next.tok;

    tok
  }

  /// Expects that the current token is t. Bumps on success.
  fn expect(&mut self, t: &token::Token) -> bool {
    if self.token == *t {
      self.bump();
      true
    } else {
      let token_str = token::to_str(t);
      let this_token_str = token::to_str(&self.token);
      self.error(format!("expected `{}` but found `{}`", token_str,
          this_token_str));
      false
    }
  }

  /// Expects that the current token is IDENT, returns its string value. Bumps
  /// on success.
  fn expect_ident(&mut self) -> Option<String> {
    let tok_str = token::to_str(&self.token);
    match self.token {
      token::IDENT(_, _) => {
        self.bump();
        Some(tok_str)
      },
      _ => {
        self.error(format!("expected identifier but found `{}`", tok_str));
        None
      },
    }
  }
}
