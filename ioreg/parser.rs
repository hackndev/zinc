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
use syntax::ast::TokenTree;
use syntax::codemap::{Span, DUMMY_SP};
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
  pub fn parse_ioregs(&mut self) -> Option<Gc<node::IoReg>> {
    let mut fields: Vec<node::FieldOrPadding> = Vec::new();
    let mut failed = false;

    // First take the register set name
    let name_span = self.span;
    let name = match self.expect_ident() {
      Some(name) => name,
      None => return None,
    };

    if !self.expect(&token::COMMA){
      return None;
    }

    loop {
      if self.token == token::EOF {
        break
      }

      let field = match self.parse_field_or_padding() {
        Some(field) => field,
        None => {
          failed = true;
          self.bump();
          continue
        }
      };
      fields.push(field);

      if self.expect(&token::COMMA) {
        break;
      }
    }

    if failed {
      None
    } else {
      Some(box(GC) node::IoReg {name: name, name_span: name_span, fields: fields})
    }
  }

  fn parse_field_or_padding(&mut self) -> Option<node::FieldOrPadding> {
    match self.expect_ident() {
      // parse padding
      Some(ref s) if s.equiv(&("pad")) => {
        if !self.expect(&token::LPAREN) {
          return None;
        }
        match self.bump() {
          token::LIT_INT_UNSUFFIXED(width) => {
            if !self.expect(&token::RPAREN) {
              return None;
            }
            Some(node::Padding(width as uint))
          },
          _ => None,
        }
      },
      Some(name) => match self.parse_field(name) {
        Some(field) => Some(node::Field(field)),
        None => None,
      },
     None => None,
    }
  }

  fn parse_field(&mut self, name: String) -> Option<node::Field> {
    let name_span = self.last_span;

    // We've already parsed the name
    if !self.expect(&token::COLON) {
      return None;
    }

    let read_only = match self.token {
      token::NOT => {
        self.bump();
        true
      },
      ref t => false,
    };

    let field_type = match self.parse_field_type() {
      Some(ty) => ty,
      None => return None,
    };

    let (width, width_span) = match self.token {
      token::LPAREN => {
        self.bump();
        match self.bump() {
          token::LIT_INT_UNSUFFIXED(width) => {
            if !self.expect(&token::RPAREN) {
              return None;
            }
            (width as uint, self.span)
          },
          _ => { return None; }
        }
      },
      _ => (1, DUMMY_SP), // default width
    };

    let (count, count_span) = match self.token {
      token::LBRACKET => {
        self.bump();
        match self.bump() {
          token::LIT_INT_UNSUFFIXED(count) => {
            if !self.expect(&token::RBRACKET) {
              return None;
            }
            (count as uint, self.span)
          },
          _ => { return None; }
        }
      },
      _ => (1, DUMMY_SP), // default repeat
    };

    let field: node::Field = node::Field { name: name, name_span: name_span,
                                           read_only: read_only,
                                           ty: field_type,
                                           width: width, width_span: width_span,
                                           count: count, count_span: count_span };
    Some(field)
  }

  fn parse_field_type(&mut self) -> Option<node::FieldType> {
    match self.expect_ident() {
      Some(ref s) if s.equiv(&("struct")) => {
        let mut fields: Vec<node::FieldOrPadding> = Vec::new();
        let ty_name = match self.token {
          ref mut t@token::IDENT(_, _) => Some(token::to_str(t)),
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
          match self.parse_field_or_padding() {
            Some(field) => fields.push(field),
            None => return None,
          }
          // FIXME: trailing comma
          if !self.expect(&token::COMMA) {
            return None;
          }
        }
        Some(node::StructType(ty_name, fields))
      },

      Some(ref s) if s.equiv(&("enum")) => {
        let mut values: Vec<node::EnumValue> = Vec::new();
        let (width, _) = match self.expect_width() {
          Some(width) => width,
          None => return None,
        };

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

          // FIXME
          if !self.expect(&token::COMMA) {
            return None;
          }
        }
        Some(node::EnumType(ty_name, values, width))
      },
      Some(ref s) if s.equiv(&("uint")) => {
        match self.expect_width() {
          Some((width,_)) => Some(node::UIntType(width)),
          _ => return None,
        }
      },
      Some(ref s) if s.equiv(&("u8"))  => Some(node::UIntType(8)),
      Some(ref s) if s.equiv(&("u16")) => Some(node::UIntType(16)),
      Some(ref s) if s.equiv(&("u32")) => Some(node::UIntType(32)),
      Some(ref s) if s.equiv(&("bool")) => Some(node::UIntType(1)),
      Some(ref s) => {
        self.error(format!("unsupported field type `{}`", s));
        return None;
      },
      None => return None,
    }
  }

  fn expect_width(&mut self) -> Option<(uint, Span)> {
    if !self.expect(&token::LPAREN) {
      return None;
    }
    let ret = match self.token {
      token::LIT_INT_UNSUFFIXED(width) => {
        self.bump();
        (width as uint, self.span)
      },
      _ => return None,
    };
    if !self.expect(&token::RPAREN) {
      return None;
    }
    Some(ret)
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
