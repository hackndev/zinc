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

use std::collections::HashMap;
use std::rc::{Rc, Weak};
use syntax::ast::{TokenTree, LitInt, UnsuffixedIntLit};
use syntax::codemap::{Span, mk_sp};
use syntax::ext::base::ExtCtxt;
use syntax::parse::{token, ParseSess, lexer, integer_lit};
use syntax::print::pprust;

use node;

pub struct Parser<'a> {
  pub sess: &'a ParseSess,
  reader: Box<lexer::Reader+'a>,
  token: token::Token,
  span: Span,

  last_token: Option<Box<token::Token>>,
  last_span: Span,
}

impl<'a> Parser<'a> {
  pub fn new(cx: &'a ExtCtxt, tts: &[TokenTree]) -> Parser<'a> {
    let sess = cx.parse_sess();
    let ttsvec = tts.iter().map(|x| (*x).clone()).collect();
    let mut reader = Box::new(lexer::new_tt_reader(
        &sess.span_diagnostic, None, None, ttsvec)) as Box<lexer::Reader>;

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
  pub fn parse_platformtree(&mut self) -> Option<Rc<node::PlatformTree>> {
    let mut nodes: HashMap<String, Rc<node::Node>> = HashMap::new();
    let mut failed = false;
    loop {
      if self.token == token::Eof {
        break
      }

      let node = match self.parse_node(None) {
        Some(node) => node,
        None => {
          failed = true;
          self.bump();
          continue
        }
      };

      let path = node.path.clone();
      if nodes.contains_key(&path) {
        failed = true;
        self.sess.span_diagnostic.span_err(node.path_span,
            format!("duplicate node definition `{}`", path).as_slice());
        let old_node: &Rc<node::Node> = &nodes[path];
        self.sess.span_diagnostic.span_err(old_node.path_span,
            "previously defined here");
      } else {
        nodes.insert(node.path.clone(), node);
      }
    }

    if failed {
      None
    } else {
      let mut map = HashMap::new();
      if self.collect_node_names(&mut map, &nodes) {
        Some(Rc::new(node::PlatformTree::new(nodes, map)))
      } else {
        None
      }
    }
  }

  fn collect_node_names(&self, map: &mut HashMap<String, Weak<node::Node>>,
      nodes: &HashMap<String, Rc<node::Node>>) -> bool {
    for (_, n) in nodes.iter() {
      let mut strongmap = HashMap::new();
      n.with_subnodes_map(|sub|{
        for (k, weak_node) in sub.iter() {
          let node = weak_node.upgrade().unwrap();
          strongmap.insert(k.clone(), node);
        }
      });

      if !self.collect_node_names(map, &strongmap) {
        return false;
      }

      match n.name {
        Some(ref name) => {
          if map.contains_key(name) {
            self.sess.span_diagnostic.span_err(n.name_span, format!(
                "duplicate `{}` definition", name).as_slice());

            self.sess.span_diagnostic.span_warn(
                (*map)[*name].upgrade().unwrap().name_span,
                "previously defined here");
            return false;
          } else {
            map.insert(name.clone(), n.downgrade());
          }
        },
        None => (),
      }
    }
    true
  }

  fn parse_node(&mut self, parent: Option<Weak<node::Node>>)
      -> Option<Rc<node::Node>> {
    use syntax::parse::token::Token;
    let name_span: Option<Span>;
    let node_name: Option<String>;

    //  we're here
    // /
    // v
    // NAME @ PATH { ... }
    //      ^-- peeking here
    if self.reader.peek().tok == token::At {
      name_span = Some(self.span);
      node_name = match self.expect_ident() {
        Some(name) => Some(name),
        None => return None,
      };
      if !self.expect(&token::At) {
        return None;
      }
    } else {
      node_name = None;
      name_span = None;
    }

    let node_span: Span;
    let node_path_span: Span;
    let attributes: HashMap<String, Rc<node::Attribute>>;
    let subnodes: node::Subnodes;

    // NAME is resolved, if it was there anyway.
    //    we're here
    //          |
    //          v
    //   NAME @ PATH { ... }
    node_path_span = self.span;
    if node_name == None {
      node_span = self.span;
    } else {
      node_span = mk_sp(name_span.unwrap().lo, self.span.hi);
    }

    let node_path = match self.token {
      Token::Ident(_, _) => {
        pprust::token_to_string(&self.bump())
      },
      Token::Literal(token::Lit::Integer(intname), _) => {
        self.bump();

        let lit = integer_lit(intname.as_str(), None, &self.sess.span_diagnostic, self.span);
        match lit {
          LitInt(i, _) => {
            format!("{}", i)
          },
          _ => {
            self.error(format!("expected unsuffixed integer but found `{}`",
                pprust::token_to_string(&self.token)));
            return None
          }
        }
      }
      ref other => {
        self.error(format!("expected node path but found `{}`",
            pprust::token_to_string(other)));
        return None;
      }
    };

    let node = Rc::new(node::Node::new(
        node_name, node_span, node_path, node_path_span, parent));
    let weak_node = node.downgrade();

    //    we're here
    //             |
    //             v
    // NAME @ PATH { ... }
    // it's either a body, or a semicolon (no body)
    match self.bump() {
      token::OpenDelim(token::Brace) => {
        let (a, s) = match self.parse_node_body(weak_node) {
          Some((attrs, subnodes)) => (attrs, subnodes),
          // TODO(farcaller): eat everything up to '}' and continue if failed
          // we can still parse further nodes.
          None => return None,
        };
        attributes = a;
        subnodes = s;

        if !self.expect(&token::CloseDelim(token::Brace)) {
          return None;
        }
      },
      token::Semi => {
        attributes = HashMap::new();
        subnodes = node::Subnodes::new();
      },
      ref other => {
        self.error(format!("expected `{{` or `;` but found `{}`",
            pprust::token_to_string(other)));
        return None;
      }
    }

    node.attributes.borrow_mut().clone_from(&attributes);
    node.set_subnodes(subnodes);
    Some(node)
  }

  fn parse_node_body(&mut self, weak_node: Weak<node::Node>)
      -> Option<(HashMap<String, Rc<node::Attribute>>, node::Subnodes)> {
    let mut attrs = HashMap::new();
    let mut subnodes = node::Subnodes::new();

    loop {
      // break early if at brace
      if self.token == token::CloseDelim(token::Brace) {
        break;
      }

      // we're here
      // |
      // v
      // ATTR = VAL ;
      // NAME @ PATH
      // PATH { ... }
      // PATH ;
      //      ^-- peeking here
      if self.reader.peek().tok == token::Eq {
        // we're here
        // |
        // v
        // ATTR = VAL ;
        let name_span = self.span;
        let some_name = match self.expect_ident() {
          Some(name) => name,
          None => return None,
        };

        if attrs.contains_key(&some_name) {
          self.error(format!("key `{}` is already defined", some_name));
          return None;
        }

        self.bump(); // bump token::Eq

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
        if !self.expect(&token::Semi) {
          return None;
        }

        attrs.insert(some_name, Rc::new(node::Attribute::new(
            attr_value, name_span, attr_value_span)));
      } else {
        // this should be a subnode
        let oldsp = self.span;
        let oldtok = self.token.clone();
        let node = match self.parse_node(Some(weak_node.clone())) {
          Some(node) => node,
          None => {
            self.span = oldsp;
            self.error(format!("expected `=` or node but found `{}`",
                pprust::token_to_string(&oldtok)));
            return None;
          },
        };

        let path = node.path.clone();
        if subnodes.as_map().contains_key(&path) {
          self.span = node.path_span;
          self.error(format!("duplicate node definition `{}`",
              path));
          let old_node: Rc<node::Node> = subnodes.as_map()[path].upgrade().unwrap();
          self.span = old_node.path_span.clone();
          self.error("previously defined here".to_string());
          return None;
        } else {
          subnodes.push(node);
        }
      }
    }

    Some((attrs, subnodes))
  }

  /// Get an Option<AttributeValue> depending on the Token type and value of
  /// self.token
  fn parse_attribute_value(&mut self) -> Option<node::AttributeValue> {
    use syntax::parse::token::{Token, Lit};
    match self.token {
      Token::Literal(Lit::Str_(string_val), _) => {
        self.bump();
        Some(node::StrValue(string_val.as_str().to_string()))
      },
      Token::Literal(Lit::Integer(intname), suffix) => {
        if suffix.is_none() {
          let lit = integer_lit(intname.as_str(), None, &self.sess.span_diagnostic, self.span);
          match lit {
            LitInt(i, UnsuffixedIntLit(_)) => {
              self.bump();
              Some(node::IntValue(i as usize))
            },
            _ => {
              self.error(format!("expected unsuffixed positive integer but found `{}`",
                  pprust::token_to_string(&self.token)));
              None
            }
          }
        } else {
          self.error(format!("expected unsuffixed integer but found `{}`",
              pprust::token_to_string(&self.token)));
          None
        }
      },
      Token::BinOp(token::And) => {
        self.bump();
        let name = match self.expect_ident() {
          Some(name) => name,
          None => return None,
        };
        Some(node::RefValue(name))
      },
      token::Ident(ident, _) => {
        self.bump();
        match &*token::get_ident(ident) {
          "true"  => Some(node::BoolValue(true)),
          "false" => Some(node::BoolValue(false)),
          other   => {
            self.error(format!(
              "expected attribute value but found identifier `{}`",
              other));
            None
          }
        }
      },
      ref other => {
        self.error(format!("expected attribute value but found `{}`",
            pprust::token_to_string(other)));
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
    self.last_token = Some(Box::new(tok.clone()));

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
      let token_str = pprust::token_to_string(t);
      let this_token_str = pprust::token_to_string(&self.token);
      self.error(format!("expected `{}` but found `{}`", token_str,
          this_token_str));
      false
    }
  }

  /// Expects that the current token is IDENT, returns its string value. Bumps
  /// on success.
  fn expect_ident(&mut self) -> Option<String> {
    let tok_str = pprust::token_to_string(&self.token);
    match self.token {
      token::Ident(_, _) => {
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
