// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

use std::gc::{Gc, GC};
use std::collections::hashmap::HashMap;
use syntax::ast::TokenTree;
use syntax::codemap::{Span, Spanned, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::parse::{token, ParseSess, lexer};

use node;

pub struct Parser<'a> {
  pub sess: &'a ParseSess,
  reader: Box<lexer::Reader>,
  token: token::Token,
  span: Span,

  last_token: Option<Box<token::Token>>,
  last_span: Span,
}

impl<'a> Parser<'a> {
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

  /// Parse the ioregs from passed in tokens.
  pub fn parse_ioregs(&mut self, cx: &ExtCtxt) -> Option<HashMap<String, Gc<node::RegGroup>>> {
    let mut groups: HashMap<String, Gc<node::RegGroup>> = HashMap::new();
    let mut failed: bool = false;

    loop {
      if self.token == token::EOF {
        break
      }
      match self.parse_reg_group(cx) {
        Some(group) => {
          groups.insert(group.name.clone(), box(GC) group);
        },
        None => {
          self.bump();
          failed = true;
          continue;
        },
      }
    }

    if failed {
      None
    } else {
      Some(groups)
    }
  }

  fn parse_reg_group(&mut self, cx: &ExtCtxt) -> Option<node::RegGroup> {
    // sitting at `group` token
    match self.expect_ident() {
      Some(ref s) if s.equiv(&"group") => {},
      _ => {
        self.error(String::from_str("Expected token `group`"));
        return None;
      },
    }

    let name_span = self.span;
    let name = match self.expect_ident() {
      Some(name) => name,
      None => return None,
    };
    if !self.expect(&token::LBRACE) {
      return None;
    }

    let mut regs: Vec<node::Reg> = Vec::new();
    let mut cur_reg: Option<node::Reg> = None;
    let mut groups: HashMap<String, node::RegGroup> = HashMap::new();
    loop {
      match self.token.clone() {
        // Beginning of register
        token::AT => {
          match cur_reg {
            None => {},
            Some(reg) => regs.push(reg),
          };
          match self.parse_reg() {
            None => return None,
            Some(reg) => cur_reg = Some(reg),
          }
        },

        // Field
        token::LPAREN =>  {
          match cur_reg {
            None => self.error(String::from_str("Found field without register")),
            Some(ref mut reg) => {
              match self.parse_field() {
                Some(field) => reg.fields.push(field),
                None => return None,
              }
            },
          }
        },

        // End of group
        token::RBRACE => {
          self.bump();
          break;
        },

        // Beginning of new group
        ref t@token::IDENT(_,_) if token::to_str(t).equiv(&"group") => {
          // Flush current register
          match cur_reg {
            None => {},
            Some(reg) => regs.push(reg),
          };
          cur_reg = None;

          match self.parse_reg_group(cx) {
            Some(group) => groups.insert(group.name.clone(), group),
            None => return None,
          };
        },

        // Error
        _ => {
          self.error(format!("fail: {}", token::to_str(&self.token)));
          self.bump();
        }
      }
    }

    let group = node::RegGroup {
      name: name,
      name_span: name_span,
      regs: regs,
      groups: groups,
    };
    Some(group)
  }

  /// Parse the introduction of a register
  fn parse_reg(&mut self) -> Option<node::Reg> {
    // we are still sitting at `@`
    self.bump();
    let offset = match self.expect_uint() {
      Some(offset) => offset,
      None => return None,
    };
    let (name, name_span) = match self.expect_ident() {
      Some(name) => (name, self.span),
      None => return None,
    };
    if !self.expect(&token::COLON) {
      return None;
    }
    let ty = match self.token.clone() {
      ref t@token::IDENT(s,_) => {
        let ty = match token::to_str(t) {
          ref s if s.equiv(&"u32") => node::UIntReg(32),
          ref s if s.equiv(&"u16") => node::UIntReg(16),
          ref s if s.equiv(&"u8")  => node::UIntReg(8),
          s                    => node::GroupReg(s),
        };
        self.bump();
        ty
      },
      ref t => {
        self.error(format!("Expected register type, found `{}`", token::to_str(t)));
        return None;
      },
    };
    let count = match self.parse_count() {
      None => return None,
      Some(count) => count,
    };

    let docstring = match self.token {
      token::LIT_STR(docstring) => {
        self.bump();
        Some(Spanned {node: docstring.to_str(), span: self.last_span})
      },
      _ => None,
    };
    Some(node::Reg {
      name: name,
      name_span: name_span,
      ty: ty,
      count: count,
      fields: Vec::new(),
      docstring: docstring,
    })
  }

  /// Parse a field
  fn parse_field(&mut self) -> Option<node::Field> {
    // sitting at ( token of bit range
    if !self.expect(&token::LPAREN) {
      return None;
    }
    let start_bit = match self.expect_uint() {
      Some(bit) => bit,
      None => return None,
    };
    let bits_span = self.span;
    let end_bit = match self.token {
      token::DOTDOT => {
        self.bump();
        match self.expect_uint() {
          Some(bit) => bit as uint,
          None => return None,
        }
      },
      _ => start_bit as uint,
    };
    if !self.expect(&token::RPAREN) {
      return None;
    }

    let name = match self.expect_ident() {
      Some(name) => Spanned {node: name, span: self.last_span},
      None => return None,
    };
    if !self.expect(&token::COLON) {
      return None;
    }
    let read_only = match self.token {
      token::NOT => {
        self.bump();
        true
      },
      _ => false,
    };
    let ty = match self.parse_field_type() {
      Some(ty) => Spanned {node: ty, span: self.last_span},
      None => return None,
    };
    let count = match self.parse_count() {
      Some(count) => count,
      None => return None,
    };
    let docstring = match self.token {
      token::LIT_STR(docstring) => {
        self.bump();
        Some(Spanned {node: docstring.to_str(), span: self.last_span})
      },
      _ => None,
    };
    let field = node::Field {
      name: name,
      bits: Spanned {node: (start_bit, end_bit), span: bits_span},
      read_only: read_only,
      ty: ty,
      count: count,
      docstring: docstring,
    };
    Some(field)
  }

  fn parse_field_type(&mut self) -> Option<node::FieldType> {
    match self.expect_ident() {
      Some(ref s) if s.equiv(&("enum")) => {
        let mut values: Vec<node::EnumValue> = Vec::new();

        let ty_name = match self.token {
          ref mut t@token::IDENT(_,_) => Some(token::to_str(t)),
          _ => None
        };

        if !self.expect(&token::LBRACE) {
          return None;
        }
        loop {
          if self.token == token::RBRACE {
            self.bump();
            break;
          }

          let (name, name_span) = match self.expect_ident() {
            Some(name) => (name, self.span),
            None => return None,
          };

          if !self.expect(&token::EQ) {
            return None;
          }

          let (value, value_span) = match self.bump() {
            token::LIT_INT_UNSUFFIXED(v) => (v as uint, self.span),
            _ => return None,
          };

          let value: node::EnumValue = node::EnumValue { name: name, name_span: name_span,
                                                         value: value, value_span: value_span };
          values.push(value);

          // FIXME: trailing comma
          if !self.expect(&token::COMMA) {
            return None;
          }
        }
        Some(node::EnumField(ty_name, values))
      },
      Some(ref s) if s.equiv(&("uint")) => Some(node::UIntField),
      Some(ref s) if s.equiv(&("bool")) => Some(node::BoolField),
      Some(s) => {
        self.error(format!("Unsupported register field type `{}`", s));
        return None;
      },
      None => return None,
    }
  }

  fn parse_uint(&mut self) -> Option<uint> {
    match self.token {
      token::LIT_INT_UNSUFFIXED(n) => {
        self.bump();
        Some(n as uint)
      },
      _ => None,
    }
  }

  fn expect_uint(&mut self) -> Option<uint> {
    match self.parse_uint() {
      Some(n) => Some(n),
      None => {
        let this_token_str = token::to_str(&self.token);
        self.error(format!("expected integer but found `{}`", this_token_str));
        None
      },
    }
  }

  fn parse_count(&mut self) -> Option<Spanned<uint>> {
    match self.token {
      token::LBRACKET => {
        self.bump();
        let ret = match self.expect_uint() {
          Some(count) => Spanned {node: count, span: self.last_span},
          None => return None,
        };
        if !self.expect(&token::RBRACKET) {
          return None;
        }
        Some(ret)
      },
      _ => Some(Spanned {node: 1, span: DUMMY_SP}),
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
