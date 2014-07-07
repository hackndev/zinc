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
          groups.insert(group.name.node.clone(), box(GC) group);
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
    let mut groups: HashMap<String, Gc<node::RegGroup>> = HashMap::new();
    loop {
      match self.token.clone() {
        // End of group
        token::RBRACE => {
          self.bump();
          break;
        },

        // Beginning of new group
        ref t@token::IDENT(_,_) if token::to_str(t).equiv(&"group") => {
          match self.parse_reg_group(cx) {
            Some(group) => groups.insert(group.name.node.clone(), box(GC) group),
            None => return None,
          };
        },

        // Presumably a register
        _ => {
          match self.parse_reg(&groups) {
            None => return None,
            Some(reg) => regs.push(reg)
          }
        },
      }
    }

    let group = node::RegGroup {
      name: Spanned {node: name, span: name_span},
      regs: regs,
      groups: groups,
    };
    Some(group)
  }

  /// Parse the introduction of a register
  fn parse_reg(&mut self, known_groups: &HashMap<String, Gc<node::RegGroup>>) -> Option<node::Reg> {
    // we are still sitting at the offset
    let offset = match self.expect_uint() {
      Some(offset) => offset,
      None => return None,
    };
    if !self.expect(&token::FAT_ARROW) {
      return None;
    }
    let name = match self.expect_ident() {
      Some(name) => Spanned {node: name, span: self.span},
      None => return None,
    };
    if !self.expect(&token::COLON) {
      return None;
    }
    let ty = match self.token.clone() {
      ref t@token::IDENT(_,_) => {
        let ty = match token::to_str(t) {
          ref s if s.equiv(&"u32") => node::U32Reg,
          ref s if s.equiv(&"u16") => node::U16Reg,
          ref s if s.equiv(&"u8")  => node::U8Reg,
          s                        => {
            match known_groups.find(&s) {
              Some(&group) => node::GroupReg(group),
              None => {
                self.error(format!("Undefined register group `{}`", s));
                return None;
              }
            }
          }
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
        Some(Spanned {node: docstring, span: self.last_span})
      },
      _ => None,
    };

    let fields = match self.token {
      // Field list
      token::LBRACE => {
        self.bump();
        let mut fields: Vec<node::Field> = Vec::new();
        loop {
          if self.token == token::RBRACE {
            self.bump();
            break;
          }
          
          match self.parse_field() {
            None => return None,
            Some(field) => fields.push(field),
          }
        }
        fields
      },
      _ => Vec::new(),
    };

    Some(node::Reg {
      offset: offset,
      name: name,
      ty: ty,
      count: count,
      fields: fields,
      docstring: docstring,
    })
  }

  /// Parse a field
  fn parse_field(&mut self) -> Option<node::Field> {
    // sitting at starting bit number
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
    if !self.expect(&token::FAT_ARROW) {
      return None;
    }

    let name = match self.expect_ident() {
      Some(name) => Spanned {node: name, span: self.last_span},
      None => return None,
    };
    if !self.expect(&token::COLON) {
      return None;
    }
    let access = match self.token.clone() {
      ref t@token::IDENT(s,_) => {
        match token::to_str(t) {
          ref s if s.equiv(&"rw") => { self.bump(); node::ReadWrite },
          ref s if s.equiv(&"ro") => { self.bump(); node::ReadOnly  },
          ref s if s.equiv(&"wo") => { self.bump(); node::WriteOnly },
          _ => node::ReadWrite,
        }
      },
      _ => node::ReadWrite,
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
        Some(Spanned {node: docstring, span: self.last_span})
      },
      _ => None,
    };
    let field = node::Field {
      name: name,
      bits: Spanned {node: (start_bit, end_bit), span: bits_span},
      access: access,
      ty: ty,
      count: count,
      docstring: docstring,
    };
    Some(field)
  }

  fn parse_field_type(&mut self) -> Option<node::FieldType> {
    match self.expect_ident() {
      Some(ref s) if s.equiv(&("enum")) => {
        let mut variants: Vec<node::Variant> = Vec::new();

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

          let name = match self.expect_ident() {
            Some(name) => Spanned {node: name, span: self.span },
            None => return None,
          };

          if !self.expect(&token::EQ) {
            return None;
          }

          let value = match self.bump() {
            token::LIT_INT_UNSUFFIXED(v) => Spanned { node: v as uint, span: self.span },
            _ => return None,
          };

          let value: node::Variant = node::Variant { name: name, value: value };
          variants.push(value);

          // FIXME: trailing comma
          if !self.expect(&token::COMMA) {
            return None;
          }
        }
        Some(node::EnumField {opt_name: ty_name, variants: variants})
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
