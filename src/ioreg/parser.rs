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

use std::rc::{Rc};
use syntax::ast::{Ident, TokenTree};
use syntax::ast;
use syntax::codemap::{Span, Spanned, respan, dummy_spanned, mk_sp};
use syntax::ext::base::ExtCtxt;
use syntax::parse::{token, ParseSess, lexer};
use syntax::parse;
use syntax::print::pprust;

use node;
use node::RegType;

/// The scope of a doc comment
enum Scope {
  /// Applies to the next item in the block (///)
  Inner,
  /// Applies to the previous item in the block (//=)
  Trailing,
  /// Applies to the owner of the current block (//!)
  Outer,
}

pub struct Parser<'a> {
  cx: &'a ExtCtxt<'a>,
  sess: &'a ParseSess,
  reader: Box<lexer::Reader+'a>,
  token: token::Token,
  span: Span,

  last_token: Option<Box<token::Token>>,
  last_span: Span,
}

impl<'a> Parser<'a> {
  pub fn new(cx: &'a ExtCtxt<'a>, tts: &[TokenTree]) -> Parser<'a> {
    let sess = cx.parse_sess();
    let ttsvec = tts.iter().map(|x| (*x).clone()).collect();
    let mut reader = Box::new(lexer::new_tt_reader(
        &sess.span_diagnostic, None, None, ttsvec)) as Box<lexer::Reader>;

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
  pub fn parse_ioregs(&mut self) -> Option<Rc<node::Reg>> {
    let name = match self.expect_ident() {
      Some(name) => respan(self.last_span, name),
      None => return None,
    };

    if !self.expect(&token::Eq) {
      return None;
    }

    let sp_lo = self.span.lo;
    if !self.expect(&token::OpenDelim(token::Brace)) {
      return None;
    }

    let docstring = self.parse_docstring(Scope::Inner);

    let regs = match self.parse_regs() {
      Some(regs) => regs,
      None => return None,
    };

    let group = node::Reg {
      offset: 0,
      name: name,
      ty: RegType::RegUnion(Rc::new(regs)),
      count: respan(mk_sp(sp_lo, self.span.hi), 1),
      docstring: docstring,
    };

    Some(Rc::new(group))
  }

  /// Parse a block of regs
  fn parse_regs(&mut self) -> Option<Vec<node::Reg>> {
    // sitting at start of first register, after LBRACE so that the
    // owner of this block can catch its docstrings

    let mut regs: Vec<node::Reg> = Vec::new();
    loop {
      match self.token.clone() {
        // End of block
        token::CloseDelim(token::Brace) => {
          self.bump();

          // Eat optional comma after closing brace
          if self.token == token::Comma {
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
    // We might have an outer docstring
    let docstring = self.parse_docstring(Scope::Outer);

    // we are still sitting at the offset
    let offset = match self.expect_uint() {
      Some(offset) => offset,
      None => return None,
    };
    if !self.expect(&token::FatArrow) {
      return None;
    }

    let ty = match self.expect_ident() {
      Some(ref i) if i.eq(&"reg32") => RegType::RegPrim(node::RegWidth::Reg32, Vec::new()),
      Some(ref i) if i.eq(&"reg16") => RegType::RegPrim(node::RegWidth::Reg16, Vec::new()),
      Some(ref i) if i.eq(&"reg8")  => RegType::RegPrim(node::RegWidth::Reg8, Vec::new()),
      Some(ref i) if i.eq(&"group") => {
        // registers will get filled in later
        RegType::RegUnion(Rc::new(Vec::new()))
      },
      _ => {
        self.error(format!("expected register type but found `{}`",
                           pprust::token_to_string(&self.token)));
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

    // Potentially a trailing docstring before the block
    let docstring = docstring.or_else(|| self.parse_docstring(Scope::Trailing));

    // Catch beginning of block and potentially an inner docstring
    if !self.expect(&token::OpenDelim(token::Brace)) {
      return None;
    }
    let docstring = docstring.or_else(|| self.parse_docstring(Scope::Inner));

    let ty = match ty {
      RegType::RegPrim(width, _) => {
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

            RegType::RegPrim(width, fields)
          },
        }
      },
      RegType::RegUnion(_) => {
        match self.parse_regs() {
          Some(regs) => RegType::RegUnion(Rc::new(regs)),
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
      if self.token == token::CloseDelim(token::Brace) {
        self.bump();

        // Eat optional comma after closing brace
        if self.token == token::Comma {
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
    // potentially an initial outer docstring
    let docstring = self.parse_docstring(Scope::Outer);

    // sitting at starting bit number
    let low_bit = match self.expect_uint() {
      Some(bit) => bit,
      None => return None,
    };
    let bits_span = self.span;
    let high_bit = match self.token {
      token::DotDot => {
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

    if !self.expect(&token::FatArrow) {
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
      token::Colon => {
        self.bump();
        match self.token.clone() {
          ref t@token::Ident(_,_) => {
            match pprust::token_to_string(t) {
              ref s if s.eq(&"rw") => { self.bump(); node::Access::ReadWrite },
              ref s if s.eq(&"ro") => { self.bump(); node::Access::ReadOnly  },
              ref s if s.eq(&"wo") => { self.bump(); node::Access::WriteOnly },
              ref s if s.eq(&"set_to_clear") => { self.bump(); node::Access::SetToClear },
              s => {
                self.error(format!("Expected access type, saw `{}`", s));
                return None;
              },
            }
          },
          ref t => {
            self.error(format!("Expected access type, saw `{}`",
                               pprust::token_to_string(t)));
            return None;
          },
        }
      },
      _ => node::Access::ReadWrite,
    };

    let (docstring, ty) = match self.token {
      token::Comma | token::CloseDelim(token::Brace) => {
        if self.token == token::Comma {
          self.bump();
        }
        let docstring = docstring.or_else(|| self.parse_docstring(Scope::Trailing));
        let ty = match width {
          1 => node::FieldType::BoolField,
          _ => node::FieldType::UIntField,
        };
        (docstring, respan(name.span, ty))
      },
      // A list of enumeration variants
      token::OpenDelim(token::Brace) => {
        self.bump();

        let sp_lo = self.span.lo;
        let docstring = docstring.or_else(|| self.parse_docstring(Scope::Inner));
        match self.parse_enum_variants() {
          Some(variants) => {
            if self.token == token::Comma {
              self.bump();
            }
            let ty = respan(
              mk_sp(sp_lo, self.span.hi),
              node::FieldType::EnumField {opt_name: None, variants: variants});
            (docstring, ty)
          },
          None => return None,
        }
      },
      _ => {
        self.error(format!(
          "Expected `,` enumeration variant list, or `}}`, found `{}`",
          pprust::token_to_string(&self.token)));
        return None;
      },
    };

    let field = node::Field {
      name: name,
      low_bit: low_bit,
      width: width,
      count: count,
      bit_range_span: bits_span,
      access: access,
      ty: ty,
      docstring: docstring,
    };
    Some(field)
  }

  fn parse_enum_variants(&mut self) -> Option<Vec<node::Variant>> {
    // sitting at beginning of block after LBRACE
    let mut variants: Vec<node::Variant> = Vec::new();

    let mut require_comma: bool = false;
    loop {
      if self.token == token::CloseDelim(token::Brace) {
        self.bump();
        break;
      }

      if require_comma && !self.expect(&token::Comma) {
        return None;
      }
      require_comma = true;

      if self.token == token::CloseDelim(token::Brace) {
        self.bump();
        break;
      }

      let value = match self.expect_uint() {
        Some(v) => respan(self.last_span, v),
        _ => return None,
      };

      if !self.expect(&token::FatArrow) {
        return None;
      }

      let name = match self.expect_ident() {
        Some(name) => respan(self.span, name),
        None => return None,
      };

      // Catch commas before the docstring
      match self.token {
        token::Comma => {
          require_comma = false;
          self.bump();
        }
        _ => {}
      }

      let docstring = self.parse_docstring(Scope::Trailing);

      let value: node::Variant = node::Variant { name: name, value: value, docstring: docstring };
      variants.push(value);
    }
    Some(variants)
  }

  fn parse_docstring(&mut self, scope: Scope) -> Option<Spanned<Ident>> {
    let mut docs: Vec<String> = Vec::new();
    let prefix = match scope {
      Scope::Inner => "//!",
      Scope::Trailing => "//=",
      Scope::Outer => "///",
    };
    loop {
      match self.token {
        token::DocComment(docstring) => {
          let s = token::get_ident(docstring.ident());
          if !s.get().starts_with(prefix) {
            break
          }

          self.bump();
          let stripped = s.get().slice_from(prefix.len())
            .trim_left_matches(' ');
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
      token::Literal(token::Integer(n), suf) => {
        self.bump();
        let lit = parse::integer_lit(n.as_str(),
                                     suf.as_ref().map(|n| n.as_str()),
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
        let this_token_str = pprust::token_to_string(&self.token);
        self.error(format!("expected integer but found `{}`", this_token_str));
        None
      },
    }
  }

  /// `None` indicates parse failure.
  /// If no count is given, a default of 1 is used
  fn parse_count(&mut self) -> Option<Spanned<uint>> {
    match self.token {
      token::OpenDelim(token::Bracket) => {
        self.bump();
        let ret = match self.expect_uint() {
          Some(count) => respan(self.last_span, count),
          None => return None,
        };
        if !self.expect(&token::CloseDelim(token::Bracket)) {
          self.error(format!("expected `]` but found `{}`",
                             pprust::token_to_string(&self.token)));
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
