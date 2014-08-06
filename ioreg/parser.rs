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
use syntax::ast;
use syntax::ast::{Ident, TokenTree};
use syntax::codemap::{Span, Spanned, respan, dummy_spanned, mk_sp};
use syntax::ext::base::ExtCtxt;
use syntax::parse;
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
  pub fn parse_ioregs(&mut self) -> Option<Gc<node::Reg>> {
    let name = match self.expect_ident() {
      Some(name) => respan(self.last_span, name),
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

    let group = node::Reg {
      offset: 0,
      name: name,
      ty: node::RegUnion(box(GC) regs),
      count: dummy_spanned(1),
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

          // Eat optional comma after closing brace
          if self.token == token::COMMA {
            self.bump();
          }

          break;
        },

        // Presumably a register
        _ => {
          match self.parse_reg() {
            Some(reg) => regs.push(reg),
            None => return None,
          }
        },
      }
    }

    regs.sort_by(|r1,r2| r1.offset.cmp(&r2.offset));

    // Verify that registers don't overlap
    let mut failed = false;
    for (r1,r2) in regs.iter().zip(regs.iter().skip(1)) {
      if r2.offset <= r1.last_byte() {
        self.sess.span_diagnostic.span_err(
          r1.name.span,
          format!("The byte range of register ({} to {})",
                  r1.offset, r1.last_byte()).as_slice());
        self.sess.span_diagnostic.span_err(
          r2.name.span,
          format!("overlaps with the range of this register ({} to {})",
                  r2.offset, r2.last_byte()).as_slice());
        failed = true;
      }
    }

    if failed {
      None
    } else {
      Some(regs)
    }
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

    let ty = match self.expect_ident() {
      Some(ref i) if i.equiv(&"reg32") => node::RegPrim(node::Reg32, Vec::new()),
      Some(ref i) if i.equiv(&"reg16") => node::RegPrim(node::Reg16, Vec::new()),
      Some(ref i) if i.equiv(&"reg8")  => node::RegPrim(node::Reg8, Vec::new()),
      Some(ref i) if i.equiv(&"group") => {
        // registers will get filled in later
        node::RegUnion(box(GC) Vec::new())
      },
      _ => {
        self.error(format!("expected register type but found `{}`",
                           token::to_string(&self.token)));
        return None;
      },
    };

    let name = match self.expect_ident() {
      Some(name) => respan(self.last_span, name),
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
          None => return None,
          Some(mut fields) => {
            // Check for overlapping fields
            fields.sort_by(|f1,f2| f1.low_bit.cmp(&f2.low_bit));
            for (f1,f2) in fields.iter().zip(fields.iter().skip(1)) {
              if f2.low_bit <= f1.high_bit() {
                self.sess.span_diagnostic.span_err(
                  f1.bit_range_span, "The bit range of this field,".as_slice());
                self.sess.span_diagnostic.span_err(
                  f2.bit_range_span,
                  "overlaps with the bit range of this field".as_slice());
                return None;
              }
            }

            // Verify fields fit in register
            match fields.last().map(|f| f.high_bit()) {
              Some(last_bit) if last_bit >= 8*width.size() => {
                self.sess.span_diagnostic.span_err(
                  name.span,
                  format!("Width of fields ({} bits) exceeds access size of register ({} bits)",
                           last_bit+1, 8*width.size()).as_slice());
                return None;
              },
              _ => {}
            }

            node::RegPrim(width, fields)
          },
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

        // Eat optional comma after closing brace
        if self.token == token::COMMA {
          self.bump();
        }

        break;
      }

      match self.parse_field() {
        None => return None,
        Some(field) => fields.push(field),
      }
    }
    Some(fields)
  }

  /// Parse a field.
  ///
  /// `None` indicates parse failure otherwise we return whether a
  /// comma is required before the next field (as we might have
  /// already seen the comma before the docstring) in addition to the
  /// parsed field.
  ///
  fn parse_field(&mut self) -> Option<node::Field> {
    let mut require_comma: bool = true;

    // sitting at starting bit number
    let low_bit = match self.expect_uint() {
      Some(bit) => bit,
      None => return None,
    };
    let bits_span = self.span;
    let high_bit = match self.token {
      token::DOTDOT => {
        self.bump();
        match self.expect_uint() {
          Some(bit) => bit as uint,
          None => return None,
        }
      },
      _ => low_bit as uint,
    };

    // TODO(bgamari): Do we want to enforce an order here?
    let (low_bit, high_bit) =
      if high_bit < low_bit {
        (high_bit, low_bit)
      } else {
        (low_bit, high_bit)
      };

    if !self.expect(&token::FAT_ARROW) {
      return None;
    }

    let name = match self.expect_ident() {
      Some(name) => respan(self.last_span, name),
      None => return None,
    };

    let (count, width): (Spanned<uint>, uint) =
      match self.parse_count() {
        Some(count) => {
          let w = high_bit - low_bit + 1;
          if w % count.node == 0 {
            (count, w / count.node)
          } else {
            self.sess.span_diagnostic.span_err(
              mk_sp(bits_span.lo, self.last_span.hi),
              format!("Bit width ({}) not divisible by count ({})",
                      w, count.node).as_slice());
            return None;
          }
        },
        None => return None,
      };

    let access = match self.token.clone() {
      token::COLON => {
        self.bump();
        match self.token.clone() {
          ref t@token::IDENT(_,_) => {
            match token::to_string(t) {
              ref s if s.equiv(&"rw") => { self.bump(); node::ReadWrite },
              ref s if s.equiv(&"ro") => { self.bump(); node::ReadOnly  },
              ref s if s.equiv(&"wo") => { self.bump(); node::WriteOnly },
              ref s if s.equiv(&"set_to_clear") => { self.bump(); node::SetToClear },
              s => {
                self.error(format!("Expected access type, saw `{}`", s));
                return None;
              },
            }
          },
          ref t => {
            self.error(format!("Expected access type, saw `{}`",
                               token::to_string(t)));
            return None;
          },
        }
      },
      _ => node::ReadWrite,
    };

    if self.token == token::COMMA {
      self.bump();
      require_comma = false;
    }

    let docstring = self.parse_docstring();

    let ty = match self.token {
      // A list of enumeration variants
      token::LBRACE if !require_comma => {
        self.error(String::from_str("Unexpected enumeration list after comma"));
        return None;
      },
      token::LBRACE => {
        // we don't require a delimiting comma after a block
        require_comma = false;
        match self.parse_enum_variants() {
          Some(variants) => node::EnumField {opt_name: None, variants: variants},
          None => return None,
        }
      },
      _ => {
        if width == 1 {
          node::BoolField
        } else {
          node::UIntField
        }
      },
    };

    // Require a comma unless we are the last element in the block
    if self.token != token::RBRACE {
      if require_comma {
        if !self.expect(&token::COMMA) {
          return None;
        }
      } else {
        match self.token {
          token::COMMA => {self.bump();},
          _ => {}
        }
      }
    }

    let field = node::Field {
      name: name,
      low_bit: low_bit,
      width: width,
      count: count,
      bit_range_span: bits_span,
      access: access,
      ty: dummy_spanned(ty),
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

    let mut require_comma: bool = false;
    loop {
      if self.token == token::RBRACE {
        self.bump();
        break;
      }

      if require_comma && !self.expect(&token::COMMA) {
        return None;
      }
      require_comma = true;

      if self.token == token::RBRACE {
        self.bump();
        break;
      }

      let value = match self.expect_uint() {
        Some(v) => respan(self.last_span, v),
        _ => return None,
      };

      if !self.expect(&token::FAT_ARROW) {
        return None;
      }

      let name = match self.expect_ident() {
        Some(name) => respan(self.span, name),
        None => return None,
      };

      // Catch commas before the docstring
      match self.token {
        token::COMMA => {
          require_comma = false;
          self.bump();
        }
        _ => {}
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
          let s = token::get_ident(docstring.ident());
          let stripped = s.get().trim_left_chars(&['/',' ']);
          docs.push(String::from_str(stripped));
        },
        _ => break,
      }
    }
    let string = docs.connect("\n");
    let string = string.as_slice().trim();
    if !string.is_empty() {
      Some(respan(self.last_span, self.cx.ident_of(string)))
    } else {
      None
    }
  }

  fn parse_uint(&mut self) -> Option<uint> {
    match self.token {
      token::LIT_INTEGER(n) => {
        self.bump();
        let lit = parse::integer_lit(n.as_str(),
                                     &self.sess.span_diagnostic,
                                     self.span);
        match lit {
          ast::LitInt(n, _)  => Some(n as uint),
          _ => None,
        }
      },
      _ => None,
    }
  }

  fn expect_uint(&mut self) -> Option<uint> {
    match self.parse_uint() {
      Some(n) => Some(n),
      None => {
        let this_token_str = token::to_string(&self.token);
        self.error(format!("expected integer but found `{}`", this_token_str));
        None
      },
    }
  }

  /// `None` indicates parse failure.
  /// If no count is given, a default of 1 is used
  fn parse_count(&mut self) -> Option<Spanned<uint>> {
    match self.token {
      token::LBRACKET => {
        self.bump();
        let ret = match self.expect_uint() {
          Some(count) => respan(self.last_span, count),
          None => return None,
        };
        if !self.expect(&token::RBRACKET) {
          self.error(format!("expected `]` but found `{}`",
                             token::to_string(&self.token)));
          return None;
        }
        Some(ret)
      },
      _ => Some(dummy_spanned(1)),
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
      let token_str = token::to_string(t);
      let this_token_str = token::to_string(&self.token);
      self.error(format!("expected `{}` but found `{}`", token_str,
          this_token_str));
      false
    }
  }

  /// Expects that the current token is IDENT, returns its string value. Bumps
  /// on success.
  fn expect_ident(&mut self) -> Option<String> {
    let tok_str = token::to_string(&self.token);
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
