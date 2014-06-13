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

use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::parse::{token, ParseSess, lexer};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};

use std::collections::hashmap;

use node;

// A helper method for the next chunk of code
trait ToStringExp {
  fn to_stringexp(&self) -> String;
}

impl ToStringExp for ::std::string::String {
  fn to_stringexp(&self) -> String {
    // TODO(farcaller): this will break if " is present in a string.
    format!("\"{}\".to_string()", self)
  }
}

// Parser-specific extensions to node::Node
impl node::Node {

  /// Returns source representation of the node.
  pub fn to_source(&self) -> String {
    // wrap each path node into quoted string
    let mappath: Vec<String> = self.path.path.iter().map(|x| x.to_stringexp()).collect();

    // build a node::Node initialization struct
    let node_struct = format!("node::Node \\{ \
        name: {}, \
        path: node::Path \\{ absolute: {}, path: vec!({}), span: None \\}, \
        attributes: hashmap::HashMap::new(), \
        subnodes: Vec::new(), \
      \\}",
      match self.name {
        Some(ref s) => format!("Some({})", s.to_stringexp()),
        None => "None".to_string(),
      },
      self.path.absolute,
      mappath.connect(", "));

    let mut init_chunks = "".to_string();
    // for each attribute, add hash insertion code
    for (k, v) in self.attributes.iter() {
      let strinified_val = match v {
        &node::UIntValue(ref u) => format!("node::UIntValue({})", u),
        &node::StrValue(ref s)  => format!("node::StrValue({})", s.to_stringexp()),
        &node::RefValue(ref r)  => format!("node::RefValue({})", r.to_stringexp()),
      };

      init_chunks = init_chunks.append(format!("attrs.insert({}, {});",
          k.to_stringexp(), strinified_val).as_slice());
    }

    // for each subnode, add vec insertion code
    for sn in self.subnodes.iter() {
      init_chunks = init_chunks.append(format!(
          "nodes.push(box {});", sn.to_source()).as_slice());
    }

    // TODO(farcaller): this triggers unused_mut for some reason.
    // wrap the struct above into struct + init code
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

impl ToTokens for node::Node {
  /// Returns node::Node as an array of tokens. Used for quote_expr.
  fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
    (cx as &ExtParseUtils).parse_tts(self.to_source())
  }
}

/// Platform tree parser.
pub struct Parser<'a> {
  /// Tracks the parsing session.
  pub sess: &'a ParseSess,

  /// Token reader.
  reader: Box<lexer::Reader:>,

  /// The current token.
  token: token::Token,

  /// The span of current token.
  span: Span,

  /// Last visited token or None, if self.token is first token ever.
  last_token: Option<Box<token::Token>>,

  /// Last visited span.
  last_span: Span,

  /// Backlog for tokens, used as strorage if token is unbump'ed.
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

  /// Raises a fatal parsing error for current span.
  fn fatal(&self, m: &str) -> ! {
    self.sess.span_diagnostic.span_fatal(self.span, m);
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

    let next = match self.backlog.pop() {
      Some(ts) => ts,
      None => self.reader.next_token(),
    };

    self.span = next.sp;
    self.token = next.tok;

    tok
  }

  /// Un-bumps the token.
  ///
  /// Pushes the current token to backlog and restores an old one from
  /// last_token. After this, last_token and last_span are broken (same as the
  /// current token).
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

  /// Expects that the current token is t. Bumps on success.
  fn expect(&mut self, t: &token::Token) {
    if self.token == *t {
      self.bump();
    } else {
      let token_str = token::to_str(t);
      let this_token_str = token::to_str(&self.token);
      self.fatal(format!("expected `{}` but found `{}`", token_str, this_token_str).as_slice());
    }
  }

  /// Expects that the current token is IDENT. Bumps on success.
  fn expect_ident(&mut self) -> token::Token {
    match self.token {
      token::IDENT(_, _) => {
        self.bump()
      },
      _ => {
        let this_token_str = token::to_str(&self.token);
        self.fatal(format!("expected identifier but found `{}`", this_token_str).as_slice());
      },
    }
  }

  /// Makes sure there are no more tokens (we are at the end of stream).
  pub fn should_finish(&mut self) {
    if self.bump() != token::EOF {
      let this_token_str = token::to_str(&self.token);
      self.fatal(format!("trailing garbage: `{}`", this_token_str).as_slice());
    }
  }

  /// Parses a platform tree node.
  pub fn parse_node(&mut self) -> node::Node {
    let mut node = node::Node::new();
    // NODE_ID @ NODE_PATH { CONTENTS }
    // or
    //         @ NODE_PATH { CONTENTS }

    node.name = match self.token {
      // this is NODE_ID
      token::IDENT(_, _) => {
        let ret = Some(token::to_str(&self.token));
        self.bump();
        ret
      },
      // this is anonymous node
      token::AT => {
        None
      },
      _ => {
        let this_token_str = token::to_str(&self.token);
        self.fatal(format!("expected identifier or `@` but found `{}`", this_token_str).as_slice())
      },
    };

    // eat @
    self.expect(&token::AT);

    // parse node path
    node.path = self.parse_path();

    // eat {
    self.expect(&token::LBRACE);

    // parse body
    self.parse_node_contents(&mut node);

    // eat }
    self.expect(&token::RBRACE);

    node
  }

  /// Parses node contents (attributes and subnodes).
  pub fn parse_node_contents(&mut self, node: &mut node::Node) {
    let mut attrs = hashmap::HashMap::new();
    let mut nodes = Vec::new();

    loop {
      match self.token {
        // got }, so it's end of current node
        token::RBRACE => break,

        // got IDENT, so this is either attribute name or non-anonymous subnode
        token::IDENT(_, _) => {
          let name = token::to_str(&self.token);
          self.bump();

          match self.token {
            // it's attribute
            token::EQ => {
              self.bump();
              let val = self.parse_attribute_value();
              self.expect(&token::SEMI);

              attrs.insert(name, val);
            },
            // it's subnode, unbump the name and re-parse as node
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
        // got @, so this must be anonymous subnode
        token::AT => {
          let node = self.parse_node();
          nodes.push(box node);
        },
        _ => {
          let this_token_str = token::to_str(&self.token);
          self.fatal(format!("expected identifier or `\\}` but found `{}`", this_token_str).as_slice())
        }
      }
    }

    node.attributes = attrs;
    node.subnodes = nodes;
  }

  /// Parses attribute value.
  pub fn parse_attribute_value(&mut self) -> node::AttributeValue {
    let val = match self.token {
      // a string
      token::LIT_STR(sv) => {
        let val = node::StrValue(token::get_ident(sv).get().to_string());
        self.bump();
        val
      },
      // an integer
      /// TODO(farcaller): any other integers can surface here?
      token::LIT_INT_UNSUFFIXED(intval) => {
        let val = node::UIntValue(intval as uint);
        self.bump();
        val
      },
      // a reference (& + IDENT)
      token::BINOP(token::AND) => {
        self.bump();
        node::RefValue(token::to_str(&self.expect_ident()).to_string())
      },
      _ => {
        let this_token_str = token::to_str(&self.token);
        self.fatal(format!("expected integer or string but found {} `{}`", self.token, this_token_str).as_slice())
      }
    };

    val
  }

  /// Parses node path
  pub fn parse_path(&mut self) -> node::Path {
    let mut v = Vec::new();
    let mut expect_more = false;
    let mut absolute = false;
    let mut path_span = self.span.clone();

    // path starts with ::, so it's absolute
    if self.token == token::MOD_SEP {
      self.bump();
      expect_more = true;
      absolute = true;
    }

    loop {
      match self.token {
        // :: separator
        token::MOD_SEP => {
          if expect_more {
            let this_token_str = token::to_str(&self.token);
            self.fatal(format!("expected identifier but found `{}`", this_token_str).as_slice())
          }
          self.bump();
          expect_more = true;
        },
        // path component
        token::IDENT(_, _) => {
          v.push(token::to_str(&self.token));
          self.bump();
          expect_more = false;
        },
        // ints are allowed in paths as well
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
    path_span.hi = self.last_span.hi;
    node::Path {
      absolute: absolute,
      path: v,
      span: Some(path_span),
    }
  }
}
