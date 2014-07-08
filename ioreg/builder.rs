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
use std::vec;
use std::iter::FromIterator;
use syntax::ast;
use syntax::ast::{P};
use syntax::abi;
use syntax::codemap::{Spanned, mk_sp, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::owned_slice;
use syntax::parse::token;

use node;

fn no_generics() -> ast::Generics {
  ast::Generics {
    lifetimes: Vec::new(),
    ty_params: owned_slice::OwnedSlice::empty()
  }
}

enum RegOrPadding<'a> {
  /// A register
  Reg(&'a node::Reg),
  /// A given number of bytes of padding
  Pad(uint)
}

/// An iterator which takes a potentially unsorted list of registers,
/// sorts them, and adds padding to make offsets correct
struct PaddedRegsIterator<'a> {
  sorted_regs: &'a Vec<node::Reg>,
  index: uint,
  last_offset: uint,
}

impl<'a> PaddedRegsIterator<'a> {
  fn new(regs: &'a mut Vec<node::Reg>) -> PaddedRegsIterator<'a> {
    regs.sort_by(|r1,r2| r1.offset.cmp(&r2.offset));
    PaddedRegsIterator {
      sorted_regs: regs,
      index: 0,
      last_offset: 0,
    }
  }
}

impl<'a> Iterator<RegOrPadding<'a>> for PaddedRegsIterator<'a> {
  fn next(&mut self) -> Option<RegOrPadding<'a>> {
    if self.index >= self.sorted_regs.len() {
      None
    } else {
      let reg = self.sorted_regs.get(self.index);
      if reg.offset > self.last_offset {
        let pad_length = reg.offset - self.last_offset;
        self.last_offset = reg.offset + reg.size();
        Some(Pad(pad_length))
      } else {
        self.index += 1;
        self.last_offset += reg.size();
        Some(Reg(reg))
      }
    }
  }
}

pub struct Builder<'a, 'b> {
  pub cx: &'a mut ExtCtxt<'b>,
  items: Vec<Gc<ast::Item>>,
}

impl<'a, 'b> Builder<'a, 'b> {
  pub fn new<'a, 'b>(cx: &'a mut ExtCtxt<'b>) -> Builder<'a, 'b> {
    Builder {cx: cx, items: Vec::new()}
  }

  pub fn emit_items(&'a mut self, reg: Gc<node::Reg>) -> Vec<P<ast::Item>> {
    node::visit_reg(reg, &mut BuildAccessors {builder: self});
    node::visit_reg(reg, &mut BuildRegStructs {builder: self});
    node::visit_reg(reg, &mut BuildUnionTypes {builder: self});
    self.items.clone()
  }

  /// Generate a `#[name(...)]` attribute of the given type
  pub fn list_attribute(&self, name: &'static str, list: Vec<&'static str>) -> ast::Attribute {
    let words = list.move_iter().map(|word| self.cx.meta_word(DUMMY_SP, token::InternedString::new(word)));
    let allow = self.cx.meta_list(DUMMY_SP, token::InternedString::new(name),
                                  FromIterator::from_iter(words));
    self.cx.attribute(DUMMY_SP, allow)
  }

  /// Generate a `#[doc="..."]` attribute of the given type
  pub fn doc_attribute(&self, docstring: token::InternedString) -> ast::Attribute {
    let s: ast::Lit_ = ast::LitStr(docstring, ast::CookedStr);
    let attr = self.cx.meta_name_value(DUMMY_SP, token::InternedString::new("doc"), s);
    self.cx.attribute(DUMMY_SP, attr)
  }

  /// Generate an unsuffixed integer literal expression with a dummy span
  fn expr_int(&self, n: i64) -> P<ast::Expr> {
    self.cx.expr_lit(DUMMY_SP, ast::LitIntUnsuffixed(n))
  }

  pub fn push_item(&mut self, item: Gc<ast::Item>) {
    self.items.push(item);
  }

  fn primitive_type_path(&self, width: node::RegWidth) -> ast::Path {
    let name = match width {
      node::Reg8  => "u8",
      node::Reg16 => "u16",
      node::Reg32 => "u32",
    };
    self.cx.path_ident(DUMMY_SP, self.cx.ident_of(name))
  }

  /// The `Path` to the type corresponding to the primitive type of
  /// the given register
  fn reg_primitive_type_path(&self, reg: &node::Reg) -> Option<ast::Path> {
    match reg.ty {
      node::RegPrim(width, _) => Some(self.primitive_type_path(width)),
      _ => None,
    }
  }

  /// The name of the structure representing a register
  fn reg_base_type(&self, path: &Vec<String>, reg: &node::Reg) -> ast::Ident {
    self.cx.ident_of(path.clone().connect("_").as_slice())
  }

  /// Returns the type of the field representing the given register within a `RegGroup` struct
  fn reg_struct_type(&self, path: &Vec<String>, reg: &node::Reg) -> P<ast::Ty> {
    let base_ty_path = self.cx.path_ident(DUMMY_SP, self.reg_base_type(path, reg));
    let base_ty: P<ast::Ty> = self.cx.ty_path(base_ty_path, None);
    match reg.count.node {
      1 => base_ty,
      n => self.cx.ty(DUMMY_SP, ast::TyFixedLengthVec(base_ty, self.cx.expr_uint(DUMMY_SP, n))),
    }
  }

  fn field_type_path(&self, path: &Vec<String>, reg: &node::Reg, field: &node::Field)
                     -> ast::Path {
    let span = field.ty.span;
    match field.ty.node {
      node::UIntField => {
        match reg.ty {
          node::RegPrim(width, _) => self.primitive_type_path(width),
          _  => fail!("The impossible happened: a union register with fields"),
        }
      },
      node::BoolField => self.cx.path_ident(span, self.cx.ident_of("bool")),
      node::EnumField { opt_name: ref opt_name, ..} => {
        match opt_name {
          &Some(ref name) => self.cx.path_ident(span, self.cx.ident_of(name.as_slice())),
          &None => {
            let name = path.clone().append_one(field.name.node.clone()).connect("_");
            self.cx.path_ident(span, self.cx.ident_of(name.as_slice()))
          }
        }
      },
    }
  }

  /*
  fn emit_instance(&self, group: P<node::RegGroup>, name: String) -> P<ast::ForeignItem> {
    let ident = self.cx.ident_of(group.name.node.as_slice());
    let ty: P<ast::Ty> = self.cx.ty_ident(DUMMY_SP, ident);
    box(GC) ast::ForeignItem {
      ident: self.cx.ident_of(name.as_slice()),
      attrs: Vec::new(),
      node: ast::ForeignItemStatic(ty, false),
      id: ast::DUMMY_NODE_ID,
      span: DUMMY_SP,
      vis: ast::Public,
    }
  }

  fn emit_instances(&self, instances: Vec<(P<node::RegGroup>, String)>) -> P<ast::Item> {
    let fmod = ast::ForeignMod {
      abi: abi::C,
      view_items: Vec::new(),
      items: FromIterator::from_iter(instances.move_iter().map(|(group,name)| self.emit_instance(group, name))),
    };
    self.cx.item(DUMMY_SP, token::special_idents::invalid, Vec::new(), ast::ItemForeignMod(fmod))
  }
  */
}

/// A visitor to build the field accessors for primitive registers
struct BuildAccessors<'a, 'b, 'c> { builder: &'a mut Builder<'b, 'c> }

impl<'a, 'b, 'c> node::RegVisitor for BuildAccessors<'a, 'b, 'c> {
  fn visit_prim_reg<'a>(&'a mut self, path: &Vec<String>, reg: &'a node::Reg,
                        width: node::RegWidth, fields: &Vec<node::Field>) {
    let accessors: Vec<Gc<ast::Method>> =
      FromIterator::from_iter(
        fields.iter().flat_map(|f| self.build_field_accessors(path, reg, f).move_iter()));
    let impl_ = ast::ItemImpl(
      no_generics(),
      None,
      self.builder.cx.ty_path(
        self.builder.cx.path_ident(DUMMY_SP, self.builder.reg_base_type(path, reg)),
        None),
      accessors
    );
    let attrs: Vec<ast::Attribute> =
      vec!(self.builder.list_attribute("allow", vec!("non_snake_case_functions", "dead_code")));
    let item = self.builder.cx.item(DUMMY_SP, self.builder.cx.ident_of(reg.name.node.as_slice()), attrs, impl_);
    self.builder.push_item(item)
  }
}

impl<'a, 'b, 'c> BuildAccessors<'a, 'b, 'c> {
  /// Build the accessors for a field
  fn build_field_accessors(&self, path: &Vec<String>, reg: &node::Reg, field: &node::Field)
                           -> Vec<P<ast::Method>> {
    // only show the docstring for one of the accessors to avoid noise
    match field.access {
      node::ReadWrite => vec!(self.build_field_setter(path, reg, field, false),
                              self.build_field_getter(path, reg, field, true)),
      node::WriteOnly => vec!(self.build_field_setter(path, reg, field, true)),
      node::ReadOnly  => vec!(self.build_field_getter(path, reg, field, true)),
    }
  }

  /// Given an `Expr` of the given register's primitive type, return an `Expr` of the field type
  fn from_primitive(&self, reg: &node::Reg, field: &node::Field, prim: P<ast::Expr>) -> P<ast::Expr> {
    match field.ty.node {
      node::UIntField => prim,
      node::BoolField => self.builder.cx.expr_binary(DUMMY_SP, ast::BiNe, prim, self.builder.expr_int(0)),
      node::EnumField {..} => {
        let from = match reg.ty {
          node::RegPrim(width,_) =>
            match width {
              node::Reg32 => "from_u32",
              node::Reg16 => "from_u16",
              node::Reg8  => "from_u8",
            },
          _            => fail!("Can't convert group register to primitive type"),
        };
        self.builder.cx.expr_method_call(
          DUMMY_SP,
          self.builder.cx.expr_call_global(
            DUMMY_SP,
            vec!(self.builder.cx.ident_of("core"), self.builder.cx.ident_of("num"), self.builder.cx.ident_of(from)),
            vec!(prim)
          ),
          self.builder.cx.ident_of("unwrap"),
          Vec::new()
        )
      },
    }
  }

  fn mask(&self, field: &node::Field) -> P<ast::Expr> {
    self.builder.expr_int(((1i << field.width) - 1) as i64)
  }

  /// An expression for the shift of a field (including the array index if necessary)
  fn shift(&self, field: &node::Field) -> P<ast::Expr> {
    let low = self.builder.expr_int(field.low_bit as i64);
    if field.count.node > 1 {
      let idx = self.builder.cx.expr_ident(DUMMY_SP, self.builder.cx.ident_of("idx"));
      let width = self.builder.expr_int(field.width as i64);
      let idx_offset = self.builder.cx.expr_binary(DUMMY_SP, ast::BiMul, idx, width);
      self.builder.cx.expr_binary(DUMMY_SP, ast::BiAdd, low, idx_offset)
    } else {
      low
    }
  }

  fn build_field_getter(&self, path: &Vec<String>, reg: &node::Reg, field: &node::Field,
                        show_docstring: bool)
                        -> P<ast::Method> {
    let ty: P<ast::Ty> = self.builder.cx.ty_path(self.builder.field_type_path(path, reg, field), None);
    let self_arg: ast::Arg = ast::Arg::new_self(DUMMY_SP, ast::MutImmutable);
    let inputs: Vec<ast::Arg> =
      if field.count.node == 1 {
        vec!(self_arg)
      } else {
        let idx_arg: ast::Arg =
          self.builder.cx.arg(DUMMY_SP, self.builder.cx.ident_of("idx"),
                              self.builder.cx.ty_ident(DUMMY_SP, self.builder.cx.ident_of("uint")));
        vec!(self_arg, idx_arg)
      };
    let decl: P<ast::FnDecl> = self.builder.cx.fn_decl(inputs, ty);

    let value: P<ast::Expr> =
      self.builder.cx.expr_method_call(
        DUMMY_SP,
        self.builder.cx.expr_field_access(DUMMY_SP, self.builder.cx.expr_self(DUMMY_SP), self.builder.cx.ident_of("_value")),
        self.builder.cx.ident_of("get"),
        Vec::new()
      );
    let shifted_masked: P<ast::Expr> =
      self.builder.cx.expr_binary(
        DUMMY_SP,
        ast::BiBitAnd,
        self.builder.cx.expr_binary(DUMMY_SP, ast::BiShr, value, self.shift(field)),
        self.mask(field));
    let expr: P<ast::Expr> = self.from_primitive(reg, field, shifted_masked);

    let mut attrs = match field.docstring {
      Some(docstring) if show_docstring =>
        vec!(self.builder.doc_attribute(token::get_ident(docstring.node))),
      _ => Vec::new(),
    };

    let body: P<ast::Block> = self.builder.cx.block(DUMMY_SP, Vec::new(), Some(expr));
    box(GC) ast::Method {
      ident: self.builder.cx.ident_of(field.name.node.as_slice()),
      attrs: attrs,
      generics: no_generics(),
      explicit_self: Spanned {span: DUMMY_SP, node: ast::SelfRegion(None, ast::MutImmutable)},
      fn_style: ast::NormalFn,
      decl: decl,
      body: body,
      id: ast::DUMMY_NODE_ID,
      span: DUMMY_SP,
      vis: ast::Public,
    }
  }

  /// Build a setter for a field
  fn build_field_setter(&self, path: &Vec<String>, reg: &node::Reg, field: &node::Field,
                        show_docstring: bool) -> P<ast::Method> {
    let ty: P<ast::Ty> = self.builder.cx.ty_path(self.builder.field_type_path(path, reg, field), None);
    let self_arg: ast::Arg = ast::Arg::new_self(DUMMY_SP, ast::MutImmutable);
    let new_value: ast::Arg = self.builder.cx.arg(DUMMY_SP, self.builder.cx.ident_of("new_value"), ty);
    let inputs: Vec<ast::Arg> =
      if field.count.node == 1 {
        vec!(self_arg, new_value)
      } else {
        let idx_arg: ast::Arg =
          self.builder.cx.arg(DUMMY_SP, self.builder.cx.ident_of("idx"),
                              self.builder.cx.ty_ident(DUMMY_SP, self.builder.cx.ident_of("uint")));
        vec!(self_arg, idx_arg, new_value)
      };
    let decl: P<ast::FnDecl> = self.builder.cx.fn_decl(inputs, self.builder.cx.ty_nil());

    let shifted_mask: P<ast::Expr> =
      self.builder.cx.expr_binary(
        DUMMY_SP,
        ast::BiShr,
        self.mask(field),
        self.shift(field));
    let cell: P<ast::Expr> =
        self.builder.cx.expr_field_access(DUMMY_SP, self.builder.cx.expr_self(DUMMY_SP), self.builder.cx.ident_of("_value"));
    let old: P<ast::Expr> =
      self.builder.cx.expr_method_call(
        DUMMY_SP,
        cell,
        self.builder.cx.ident_of("get"),
        Vec::new()
      );
    let old_masked: P<ast::Expr> =
      self.builder.cx.expr_binary(
        DUMMY_SP,
        ast::BiBitAnd,
        old,
        self.builder.cx.expr_unary(DUMMY_SP, ast::UnNot, shifted_mask)
      );
    let new_masked_shifted: P<ast::Expr> =
      self.builder.cx.expr_binary(
        DUMMY_SP,
        ast::BiShl,
        self.builder.cx.expr_binary(
          DUMMY_SP,
          ast::BiBitAnd,
          self.builder.cx.expr_cast(
            DUMMY_SP,
            self.builder.cx.expr_ident(DUMMY_SP, self.builder.cx.ident_of("new_value")),
            self.builder.cx.ty_path(self.builder.reg_primitive_type_path(reg).unwrap(), None)),
          self.mask(field)
        ),
        self.shift(field));
    let expr: Gc<ast::Expr> =
      self.builder.cx.expr_method_call(
        DUMMY_SP,
        cell,
        self.builder.cx.ident_of("set"),
        vec!(self.builder.cx.expr_binary(DUMMY_SP, ast::BiBitOr, old_masked, new_masked_shifted)));

    let mut attrs = match field.docstring {
      Some(docstring) if show_docstring =>
        vec!(self.builder.doc_attribute(token::get_ident(docstring.node))),
      _ => Vec::new(),
    };

    let body: P<ast::Block> = self.builder.cx.block(DUMMY_SP, vec!(self.builder.cx.stmt_expr(expr)), None);
    box(GC) ast::Method {
      ident: self.builder.cx.ident_of((String::from_str("set_")+field.name.node).as_slice()),
      attrs: attrs,
      generics: no_generics(),
      explicit_self: Spanned {span: DUMMY_SP, node: ast::SelfRegion(None, ast::MutImmutable)},
      fn_style: ast::NormalFn,
      decl: decl,
      body: body,
      id: ast::DUMMY_NODE_ID,
      span: DUMMY_SP,
      vis: ast::Public,
    }
  }
}

/// A visitor to build the struct for each register
struct BuildRegStructs<'a, 'b, 'c> {builder: &'a mut Builder<'b, 'c>}

impl<'a, 'b, 'c> node::RegVisitor for BuildRegStructs<'a, 'b, 'c> {
  fn visit_prim_reg(&mut self, path: &Vec<String>, reg: &node::Reg,
                    width: node::RegWidth, fields: &Vec<node::Field>) {
    for field in fields.iter() {
      match self.build_field_type(path, reg, field) {
        Some(item) => self.builder.push_item(item),
        None       => {}
      }
    }

    let reg_struct = self.build_reg_struct(path, reg, width);
    self.builder.push_item(reg_struct);
  }
}

impl<'a, 'b, 'c> BuildRegStructs<'a, 'b, 'c> {
  /// Build a field type if necessary (e.g. in the case of an `EnumField`)
  fn build_field_type(&self, path: &Vec<String>, reg: &node::Reg, field: &node::Field)
                      -> Option<P<ast::Item>> {
    match field.ty.node {
      node::EnumField { variants: ref variants, .. } => {
        // FIXME: We construct a path, then only take the last segment, this could be more efficient
        let name: ast::Ident = self.builder.field_type_path(path, reg, field).segments.last().unwrap().identifier;
        let enum_def: ast::EnumDef = ast::EnumDef {
          variants: FromIterator::from_iter(variants.iter().map(|v| box(GC) self.build_enum_variant(v))),
        };
        let attrs: Vec<ast::Attribute> = vec!(
          self.builder.list_attribute("deriving", vec!("FromPrimitive")),
          self.builder.list_attribute("allow", vec!("uppercase_variables", "dead_code", "non_camel_case_types")));
        let item: P<ast::Item> = box(GC) ast::Item {
          ident: name,
          id: ast::DUMMY_NODE_ID,
          node: ast::ItemEnum(enum_def, no_generics()),
          vis: ast::Public,
          attrs: attrs,
          span: field.ty.span,
        };
        Some(item)
      },
      _ => None,
    }
  }

  /// Produce a register struct if necessary (in the case of primitive typed registers).
  /// In this case `None` indicates no struct is necessary, not failure.
  /// For instance,
  ///
  ///     pub struct REG {_value: u32}
  fn build_reg_struct(&self, path: &Vec<String>, reg: &node::Reg, width: node::RegWidth)
                      -> P<ast::Item> {
    let prim_ty = self.builder.cx.ty_path(self.builder.primitive_type_path(width), None);
    let cell_ty: P<ast::Ty> =
      self.builder.cx.ty_path(
        self.builder.cx.path_all(
          DUMMY_SP,
          false,
          vec!(self.builder.cx.ident_of("VolatileCell")),
          Vec::new(),
          vec!(prim_ty)
        ),
        None
      );
    let struct_def = ast::StructDef {
      fields: vec!(
        Spanned {
          span: DUMMY_SP,
          node: ast::StructField_ {
            kind: ast::NamedField(self.builder.cx.ident_of("_value"), ast::Inherited),
            id: ast::DUMMY_NODE_ID,
            ty: cell_ty,
            attrs: Vec::new(),
          },
        },
      ),
      ctor_id: None,
      super_struct: None,
      is_virtual: false,
    };
    let mut attrs = match reg.docstring {
      Some(docstring) => vec!(self.builder.doc_attribute(token::get_ident(docstring.node))),
      None => Vec::new(),
    };
    attrs.push(self.builder.list_attribute("allow", vec!("non_camel_case_types")));
    let item: P<ast::Item> = box(GC) ast::Item {
      ident: self.builder.reg_base_type(path, reg),
      attrs: attrs,
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemStruct(box(GC) struct_def, no_generics()),
      vis: ast::Public,
      span: reg.name.span
    };
    item
  }

  /// Build a variant of an `EnumField`
  fn build_enum_variant(&self, variant: &node::Variant) -> ast::Variant {
    let attrs = match variant.docstring {
      Some(docstring) => vec!(self.builder.doc_attribute(token::get_ident(docstring.node))),
      None => Vec::new(),
    };

    Spanned {
      span: mk_sp(variant.name.span.lo, variant.value.span.hi),
      node: ast::Variant_ {
        name: self.builder.cx.ident_of(variant.name.node.as_slice()),
        attrs: attrs,
        kind: ast::TupleVariantKind(Vec::new()),
        id: ast::DUMMY_NODE_ID,
        disr_expr: Some(self.builder.cx.expr_lit(
          variant.value.span,
          ast::LitIntUnsuffixed(variant.value.node as i64)
        )),
        vis: ast::Inherited,
      }
    }
  }
}

/// Build types for `RegUnions`
struct BuildUnionTypes<'a, 'b, 'c> {builder: &'a mut Builder<'b, 'c>}

impl<'a, 'b, 'c> node::RegVisitor for BuildUnionTypes<'a, 'b, 'c> {
  fn visit_union_reg<'a>(&'a mut self, path: &Vec<String>, reg: &'a node::Reg,
                         subregs: Gc<Vec<node::Reg>>) {
    let union_type = self.build_union_type(path, reg, subregs);
    self.builder.push_item(union_type);
  }
}

impl<'a, 'b, 'c> BuildUnionTypes<'a, 'b, 'c> {
  /// Produce a field for the given register in a `RegUnion` struct
  fn build_reg_union_field(&self, path: &Vec<String>, reg: &node::Reg) -> ast::StructField {
    let attrs = match reg.docstring {
      Some(doc) => vec!(self.builder.doc_attribute(token::get_ident(doc.node))),
      None => Vec::new(),
    };
    let field_path = path.clone().append_one(reg.name.node.clone());
    Spanned {
      span: DUMMY_SP,
      node: ast::StructField_ {
        kind: ast::NamedField(self.builder.cx.ident_of(reg.name.node.as_slice()), ast::Public),
        id: ast::DUMMY_NODE_ID,
        ty: self.builder.reg_struct_type(&field_path, reg),
        attrs: attrs,
      }
    }
  }

  /// Build field for padding or a register
  fn build_pad_or_reg<'a>(&self, path: &Vec<String>, regOrPad: RegOrPadding<'a>) -> ast::StructField {
   match regOrPad {
      Reg(reg) => self.build_reg_union_field(path, reg),
      Pad(length) => {
        let u8_path = self.builder.cx.path_ident(DUMMY_SP, self.builder.cx.ident_of("u8"));
        let u8_ty: P<ast::Ty> = self.builder.cx.ty_path(u8_path, None);
        let ty: P<ast::Ty> =
          self.builder.cx.ty(DUMMY_SP,
                     ast::TyFixedLengthVec(u8_ty, self.builder.cx.expr_uint(DUMMY_SP, length)));
        Spanned {
          span: DUMMY_SP,
          node: ast::StructField_ {
            kind: ast::NamedField(self.builder.cx.ident_of("padding"), ast::Inherited),
            id: ast::DUMMY_NODE_ID,
            ty: ty,
            attrs: Vec::new(),
          },
        }
      },
    }
  }

  /// Build the type associated with a register group
  fn build_union_type(&self, path: &Vec<String>, reg: &node::Reg, regs: &Vec<node::Reg>) -> P<ast::Item> {
    let name = Spanned {
      node: String::from_str(token::get_ident(self.builder.reg_base_type(path, reg)).get()),
      span: DUMMY_SP
    };
    let mut sorted_regs = regs.clone();
    let padded_regs = PaddedRegsIterator::new(&mut sorted_regs);
    let fields = padded_regs.map(|r| self.build_pad_or_reg(path, r));
    let struct_def = ast::StructDef {
      fields: FromIterator::from_iter(fields),
      ctor_id: None,
      super_struct: None,
      is_virtual: false,
    };
    let mut attrs: Vec<ast::Attribute> = vec!(
      self.builder.list_attribute("allow", vec!("non_camel_case_types", "uppercase_variables", "dead_code")),
    );
    match reg.docstring {
      Some(docstring) => attrs.push(self.builder.doc_attribute(token::get_ident(docstring.node))),
      None => (),
    }
    box(GC) ast::Item {
      ident: self.builder.cx.ident_of(name.node.as_slice()),
      attrs: attrs,
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemStruct(box(GC) struct_def, no_generics()),
      vis: ast::Public,
      span: name.span,
    }
  }
}
