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
use syntax::ast::Ident;
use syntax::ast::TokenTree;
use syntax::codemap::{Span, Spanned, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::parse::{token, ParseSess, lexer};

use node;

pub struct Parser<'a,'b> {
  cx: &'a ExtCtxt<'b>,
  sess: &'a ParseSess,
  reader: Box<lexer::Reader>,
  token: token::Token,
  span: Span,

  last_token: Option<Box<token::Token>>,
  last_span: Span,
}

impl<'a, 'b> Parser<'a, 'b> {
  pub fn new<'a, 'b>(cx: &'a ExtCtxt<'b>, tts: &[TokenTree]) -> Parser<'a, 'b> {
    let sess = cx.parse_sess();
    let ttsvec = tts.iter().map(|x| (*x).clone()).collect();
    let mut reader = box lexer::new_tt_reader(
        &sess.span_diagnostic, None, ttsvec) as Box<lexer::Reader>;

    let tok0 = reader.next_token();
    let token = tok0.tok;
    let span = tok0.sp;

    Parser {
      cx: cx,
      sess: sess,
      reader: reader,

      token: token,
      span: span,

      last_token: None,
      last_span: span,
    }
  }

  /// Parse the ioregs from passed in tokens.
  pub fn parse_ioregs(&mut self) -> Option<Gc<node::RegGroup>> {
    let name = match self.expect_ident() {
      Some(name) => Spanned {node: name, span: self.last_span},
      None => return None,
    };

    if !self.expect(&token::EQ) {
      return None;
    }

    if !self.expect(&token::LBRACE) {
      return None;
    }

    let docstring = self.parse_docstring();

    let regs = match self.parse_regs() {
      Some(regs) => regs,
      None => return None,
    };

    let group = node::RegGroup {
      name: name,
      regs: regs,
      docstring: docstring,
    };

    Some(box(GC) group)
  }

  /// Parse a block of regs
  fn parse_regs(&mut self) -> Option<Vec<node::Reg>> {
    // sitting at start of first register, after LBRACE so that the
    // owner of this block can catch its docstrings
    
    let mut regs: Vec<node::Reg> = Vec::new();
    loop {
      match self.token.clone() {
        // End of block
        token::RBRACE => {
          self.bump();
          break;
        },

        // Presumably a register
        _ => {
          match self.parse_reg() {
            None => return None,
            Some(reg) => regs.push(reg)
          }
        },
      }
    }

    Some(regs)
  }

  /// Parse the introduction of a register
  fn parse_reg(&mut self) -> Option<node::Reg> {
    // we are still sitting at the offset
    let offset = match self.expect_uint() {
      Some(offset) => offset,
      None => return None,
    };
    if !self.expect(&token::FAT_ARROW) {
      return None;
    }
    let mut ty = match self.expect_ident() {
      Some(ref i) if i.equiv(&"reg32") => node::RegPrim(node::Reg32, Vec::new()),
      Some(ref i) if i.equiv(&"reg16") => node::RegPrim(node::Reg16, Vec::new()),
      Some(ref i) if i.equiv(&"reg8")  => node::RegPrim(node::Reg8, Vec::new()),
      Some(ref i) if i.equiv(&"group") => {
        // registers will get filled in later
        node::RegUnion(box(GC) Vec::new())
      },
      _ => return None,
    };

    let name = match self.expect_ident() {
      Some(name) => Spanned {node: name, span: self.span},
      None => return None,
    };
    let count = match self.parse_count() {
      None => return None,
      Some(count) => count,
    };

    let docstring = self.parse_docstring();

    let ty = match ty {
      node::RegPrim(width, _) => {
        if !self.expect(&token::LBRACE) {
          return None;
        }
        match self.parse_fields() {
          Some(fields) => node::RegPrim(width, fields),
          None => return None,
        }
      },
      node::RegUnion(_) => {
        if !self.expect(&token::LBRACE) {
          return None;
        }
        match self.parse_regs() {
          Some(regs) => node::RegUnion(box(GC) regs),
          None => return None,
        }
      },
    };
     
    if !self.expect(&token::COMMA) {
      return None;
    }

    Some(node::Reg {
      offset: offset,
      name: name,
      ty: ty,
      count: count,
      docstring: docstring,
    })
  }

  fn parse_fields(&mut self) -> Option<Vec<node::Field>> {
    // sitting at starting bit number
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
    Some(fields)
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

    let count = match self.parse_count() {
      Some(count) => count,
      None => return None,
    };

    let access = match self.token.clone() {
      token::COLON => {
        self.bump();
        match self.token.clone() {
          ref t@token::IDENT(_,_) => {
            match token::to_str(t) {
              ref s if s.equiv(&"rw") => { self.bump(); node::ReadWrite },
              ref s if s.equiv(&"ro") => { self.bump(); node::ReadOnly  },
              ref s if s.equiv(&"wo") => { self.bump(); node::WriteOnly },
              s => {
                self.error(format!("Expected access type, saw `{}`", s));
                return None;
              },
            }
          },
          ref t => {
            self.error(format!("Expected access type, saw `{}`", token::to_str(t)));
            return None;
          },
        }
      },
      _ => node::ReadWrite,
    };

    let docstring = self.parse_docstring();

    let ty = match self.token {
      // A list of enumeration variants
      token::LBRACE => {
        match self.parse_enum_variants() {
          Some(variants) => node::EnumField {opt_name: None, variants: variants},
          None => return None,
        }
      },
      _ => {
        if end_bit == start_bit {
          node::BoolField
        } else {
          node::UIntField
        }
      },
    };

    if !self.expect(&token::COMMA) {
      return None;
    }

    let docstring = match docstring {
      None => self.parse_docstring(),
      _    => docstring,
    };

    let field = node::Field {
      name: name,
      bits: Spanned {node: (start_bit, end_bit), span: bits_span},
      access: access,
      ty: Spanned {span: DUMMY_SP, node: ty},
      count: count,
      docstring: docstring,
    };
    Some(field)
  }

  fn parse_enum_variants(&mut self) -> Option<Vec<node::Variant>> {
    // sitting on LBRACE
    let mut variants: Vec<node::Variant> = Vec::new();

    if !self.expect(&token::LBRACE) {
      return None;
    }
    loop {
      if self.token == token::RBRACE {
        self.bump();
        break;
      }

      let value = match self.token {
        token::LIT_INT_UNSUFFIXED(v) => {
          self.bump();
          Spanned { node: v as uint, span: self.span }
        },
        _ => return None,
      };

      if !self.expect(&token::FAT_ARROW) {
        return None;
      }

      let name = match self.expect_ident() {
        Some(name) => Spanned {node: name, span: self.span },
        None => return None,
      };

      // FIXME: trailing comma
      if !self.expect(&token::COMMA) {
        return None;
      }

      let docstring = self.parse_docstring();

      let value: node::Variant = node::Variant { name: name, value: value, docstring: docstring };
      variants.push(value);
    }
    Some(variants)
  }

  fn parse_docstring(&mut self) -> Option<Spanned<Ident>> {
    let mut docs: Vec<String> = Vec::new();
    loop {
      match self.token {
        token::DOC_COMMENT(docstring) => {
          self.bump();
          // for some reason ident begins with '/// '
          let s = token::get_ident(docstring);
          let stripped = s.get().trim_left_chars(&['/',' ']);
          docs.push(String::from_str(stripped));
        },
        _ => return None,
      }
    }
    let docstring = self.cx.ident_of(docs.connect("\n").as_slice());
    Some(Spanned {node: docstring, span: self.last_span})
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
