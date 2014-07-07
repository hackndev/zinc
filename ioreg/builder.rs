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
  groups: HashMap<String, Gc<node::RegGroup>>,
  cx: &'a mut ExtCtxt<'b>
}

impl<'a, 'b> Builder<'a, 'b> {
  pub fn new<'a, 'b>(cx: &'a mut ExtCtxt<'b>, groups: HashMap<String, Gc<node::RegGroup>>) -> Builder<'a, 'b> {
    Builder {
      groups: groups,
      cx: cx,
    }
  }

  /// Generate a `#[name(...)]` attribute of the given type
  fn list_attribute(&self, name: &'static str, list: Vec<&'static str>) -> ast::Attribute {
    let words = list.move_iter().map(|word| self.cx.meta_word(DUMMY_SP, token::InternedString::new(word)));
    let allow = self.cx.meta_list(DUMMY_SP, token::InternedString::new(name),
                                  FromIterator::from_iter(words));
    self.cx.attribute(DUMMY_SP, allow)
  }

  /// Generate a `#[doc="..."]` attribute of the given type
  fn doc_attribute(&self, docstring: token::InternedString) -> ast::Attribute {
    let s: ast::Lit_ = ast::LitStr(docstring, ast::CookedStr);
    let attr = self.cx.meta_name_value(DUMMY_SP, token::InternedString::new("doc"), s);
    self.cx.attribute(DUMMY_SP, attr)
  }

  /// Generate an unsuffixed integer literal expression with a dummy span
  fn expr_int(&self, n: i64) -> P<ast::Expr> {
    self.cx.expr_lit(DUMMY_SP, ast::LitIntUnsuffixed(n))
  }

  pub fn emit_items(&self) -> Vec<Gc<ast::Item>> {
    let items = self.groups.values().flat_map(|&g| self.emit_group_items(g).move_iter());
    // FIXME: implement real instance generation
    let instances: Vec<(P<node::RegGroup>, String)> =
      FromIterator::from_iter(self.groups.values().map(|&g| (g, g.name.node.clone()+"0")));
    let instances = vec!(self.emit_instances(instances)).move_iter();
    FromIterator::from_iter(items.chain(instances))
  }

  pub fn emit_group_items(&self, group: P<node::RegGroup>) -> Vec<Gc<ast::Item>> {
    let structs = self.emit_group_types(group).move_iter();
    let enums = group.regs.iter().flat_map(|r| self.emit_reg_field_types(group, r).move_iter());
    let accessors: vec::MoveItems<Gc<ast::Item>> = self.emit_group_accessors(group).move_iter();
    FromIterator::from_iter(structs.chain(enums).chain(accessors))
  }

  fn primitive_type_path(&self, reg_ty: node::RegType) -> Option<ast::Path> {
    let name = match reg_ty {
      node::U8Reg  => "u8",
      node::U16Reg => "u16",
      node::U32Reg => "u32",
      _  => return None
    };
    Some(self.cx.path_ident(DUMMY_SP, self.cx.ident_of(name)))
  }
  
  /// Returns the primitive type of the given width
  fn primitive_type(&self, reg_ty: node::RegType) -> Option<P<ast::Ty>> {
    match self.primitive_type_path(reg_ty) {
      Some(path) => Some(self.cx.ty_path(path, None)),
      None => None,
    }
  }

  /// Produce a register struct if necessary (in the case of primitive typed registers).
  /// For instance,
  ///
  ///     pub struct REG {_value: u32}
  fn emit_reg_struct(&self, group: P<node::RegGroup>, reg: &node::Reg) -> Option<P<ast::Item>> {
    match reg.ty {
      node::GroupReg(_) => None,
      _ => {
        let prim_ty = match self.primitive_type(reg.ty) {
          Some(ty) => ty,
          None => return None,
        };
        let cell_ty: P<ast::Ty> =
          self.cx.ty_path(
            self.cx.path_all(
              DUMMY_SP,
              false,
              vec!(self.cx.ident_of("VolatileCell")),
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
                kind: ast::NamedField(self.cx.ident_of("_value"), ast::Inherited),
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
          Some(docstring) => vec!(self.doc_attribute(token::get_ident(docstring.node))),
          None => Vec::new(),
        };
        attrs.push(self.list_attribute("allow", vec!("non_camel_case_types")));
        let item: P<ast::Item> = box(GC) ast::Item {
          ident: self.reg_base_type(group, reg),
          attrs: attrs,
          id: ast::DUMMY_NODE_ID,
          node: ast::ItemStruct(box(GC) struct_def, no_generics()),
          vis: ast::Public,
          span: reg.name.span
        };
        Some(item)
      },
    }
  }

  /// The name of the structure representing a register
  fn reg_base_type(&self, group: P<node::RegGroup>, reg: &node::Reg) -> ast::Ident {
    match reg.ty {
      node::GroupReg(ref g) => self.cx.ident_of(g.name.node.as_slice()),
      _ => self.cx.ident_of(format!("{}_{}", group.name.node, reg.name.node).as_slice()),
    }
  }

  /// Returns the type of the field representing the given register in a `RegGroup` struct
  fn reg_struct_type(&self, group: P<node::RegGroup>, reg: &node::Reg) -> P<ast::Ty> {
    let base_ty_path = self.cx.path_ident(DUMMY_SP, self.reg_base_type(group, reg));
    let base_ty: P<ast::Ty> = self.cx.ty_path(base_ty_path, None);
    match reg.count.node {
      1 => base_ty,
      n => self.cx.ty(DUMMY_SP, ast::TyFixedLengthVec(base_ty, self.cx.expr_uint(DUMMY_SP, n))),
    }
  }

  /// Produce a field for the given register in a `RegGroup` struct
  fn emit_reg_group_field(&self, group: P<node::RegGroup>, reg: &node::Reg) -> ast::StructField {
    let attrs = match reg.docstring {
      Some(doc) => vec!(self.doc_attribute(token::get_ident(doc.node))),
      None => Vec::new(),
    };
    Spanned {
      span: DUMMY_SP,
      node: ast::StructField_ {
        kind: ast::NamedField(self.cx.ident_of(reg.name.node.as_slice()), ast::Public),
        id: ast::DUMMY_NODE_ID,
        ty: self.reg_struct_type(group, reg),
        attrs: attrs,
      }
    }
  }

  /// Emit field for padding or a register
  fn emit_pad_or_reg<'a>(&self, group: P<node::RegGroup>, regOrPad: RegOrPadding<'a>) -> ast::StructField {
    match regOrPad {
      Reg(reg) => self.emit_reg_group_field(group, reg),
      Pad(length) => {
        let u8_path = self.cx.path_ident(DUMMY_SP, self.cx.ident_of("u8"));
        let u8_ty: P<ast::Ty> = self.cx.ty_path(u8_path, None);
        let ty: P<ast::Ty> =
          self.cx.ty(DUMMY_SP,
                     ast::TyFixedLengthVec(u8_ty, self.cx.expr_uint(DUMMY_SP, length)));
        Spanned {
          span: DUMMY_SP,
          node: ast::StructField_ {
            kind: ast::NamedField(self.cx.ident_of("padding"), ast::Inherited),
            id: ast::DUMMY_NODE_ID,
            ty: ty,
            attrs: Vec::new(),
          },
        }
      },
    }
  }

  /// Emit the types associated with a register group
  fn emit_group_types(&self, group: P<node::RegGroup>) -> Vec<P<ast::Item>> {
    let mut sorted_regs = group.regs.clone();
    let padded_regs = PaddedRegsIterator::new(&mut sorted_regs);
    let fields = padded_regs.map(|r| self.emit_pad_or_reg(group, r));
    let struct_def = ast::StructDef {
      fields: FromIterator::from_iter(fields),
      ctor_id: None,
      super_struct: None,
      is_virtual: false,
    };
    let span = DUMMY_SP; // FIXME
    let mut attrs: Vec<ast::Attribute> = vec!(
      self.list_attribute("allow", vec!("non_camel_case_types", "uppercase_variables", "dead_code")),
    );
    match group.docstring {
      Some(doc) => attrs.push(self.doc_attribute(token::get_ident(doc.node))),
      None => (),
    }
    let struct_item = box(GC) ast::Item {
      ident: self.cx.ident_of(group.name.node.as_slice()),
      attrs: attrs,
      id: ast::DUMMY_NODE_ID,
      node: ast::ItemStruct(box(GC) struct_def, no_generics()),
      vis: ast::Public,
      span: span
    };

    let reg_structs = group.regs.iter().flat_map(|r| self.emit_reg_struct(group, r).move_iter());
    let subgroups = group.groups.values().flat_map(|&g| self.emit_group_types(g).move_iter());
    let hi: Vec<P<ast::Item>> = FromIterator::from_iter(subgroups.chain(reg_structs));
    hi.append_one(struct_item)
  }

  fn field_type_path(&self, parent: P<node::RegGroup>, reg: &node::Reg, field: &node::Field)
                     -> ast::Path {
    let span = field.ty.span;
    match field.ty.node {
      node::UIntField => self.primitive_type_path(reg.ty).unwrap(),
      node::BoolField => self.cx.path_ident(span, self.cx.ident_of("bool")),
      node::EnumField { opt_name: ref opt_name, ..} => {
        match opt_name {
          &Some(ref name) => self.cx.path_ident(span, self.cx.ident_of(name.as_slice())),
          &None => {
            let name = parent.name.node + "_" + reg.name.node + "_" + field.name.node;
            self.cx.path_ident(span, self.cx.ident_of(name.as_slice()))
          }
        }
      },
    }
  }

  /// Emit a variant of an `EnumField`
  fn emit_enum_variant(&self, variant: &node::Variant) -> ast::Variant {
    let attrs = match variant.docstring {
      Some(docstring) => vec!(self.doc_attribute(token::get_ident(docstring.node))),
      None => Vec::new(),
    };

    Spanned {
      span: mk_sp(variant.name.span.lo, variant.value.span.hi),
      node: ast::Variant_ {
        name: self.cx.ident_of(variant.name.node.as_slice()),
        attrs: attrs,
        kind: ast::TupleVariantKind(Vec::new()),
        id: ast::DUMMY_NODE_ID,
        disr_expr: Some(self.cx.expr_lit(
          variant.value.span,
          ast::LitIntUnsuffixed(variant.value.node as i64)
        )),
        vis: ast::Inherited,
      }
    }
  }

  /// Emit a field type if necessary (e.g. in the case of an `EnumField`)
  fn emit_field_type(&self, parent: P<node::RegGroup>, reg: &node::Reg, field: &node::Field)
                     -> Option<P<ast::Item>> {
    match field.ty.node {
      node::EnumField { variants: ref variants, .. } => {
        // FIXME: We construct a path, then only take the last segment, this could be more efficient
        let name: ast::Ident = self.field_type_path(parent, reg, field).segments.last().unwrap().identifier;
        let enum_def: ast::EnumDef = ast::EnumDef {
          variants: FromIterator::from_iter(variants.iter().map(|v| box(GC) self.emit_enum_variant(v))),
        };
        let attrs: Vec<ast::Attribute> = vec!(
          self.list_attribute("deriving", vec!("FromPrimitive")),
          self.list_attribute("allow", vec!("uppercase_variables", "dead_code", "non_camel_case_types")));
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

  /// Emit types for the fields of a register
  pub fn emit_reg_field_types(&self, group: P<node::RegGroup>, reg: &node::Reg) -> Vec<Gc<ast::Item>> {
    let hi = reg.fields.iter().flat_map(|f| self.emit_field_type(group, reg, f).move_iter());
    FromIterator::from_iter(hi)
  }

  /// Emit a setter for a field
  fn emit_field_setter(&self, parent: P<node::RegGroup>, reg: &node::Reg, field: &node::Field)
                       -> P<ast::Method> {
    let ty: P<ast::Ty> = self.cx.ty_path(self.field_type_path(parent, reg, field), None);
    let self_arg: ast::Arg = ast::Arg::new_self(DUMMY_SP, ast::MutImmutable);
    let new_value: ast::Arg = self.cx.arg(DUMMY_SP, self.cx.ident_of("new_value"), ty);
    let inputs: Vec<ast::Arg> =
      if field.count.node == 1 {
        vec!(self_arg, new_value)
      } else {
        vec!(self_arg, new_value) // FIXME
      };
    let decl: P<ast::FnDecl> = self.cx.fn_decl(inputs, self.cx.ty_nil());

    let (lo,hi) = field.bits.node;
    let mask: uint = (1 << (hi-lo+1)) - 1;
    let cell: P<ast::Expr> = 
        self.cx.expr_field_access(DUMMY_SP, self.cx.expr_self(DUMMY_SP), self.cx.ident_of("_value"));
    let old: P<ast::Expr> =
      self.cx.expr_method_call(
        DUMMY_SP,
        cell,
        self.cx.ident_of("get"),
        Vec::new()
      );
    let old_masked: P<ast::Expr> =
      self.cx.expr_binary(
        DUMMY_SP,
        ast::BiBitAnd,
        old,
        self.cx.expr_unary(DUMMY_SP, ast::UnNot, self.expr_int((mask << lo) as i64))
      );
    let new_masked_shifted: P<ast::Expr> =
      self.cx.expr_binary(
        DUMMY_SP,
        ast::BiShl,
        self.cx.expr_binary(
          DUMMY_SP,
          ast::BiBitAnd,
          self.cx.expr_cast(
            DUMMY_SP,
            self.cx.expr_ident(DUMMY_SP, self.cx.ident_of("new_value")),
            self.primitive_type(reg.ty).unwrap()),
          self.expr_int(mask as i64)
        ),
        self.expr_int(lo as i64)
      );
    let expr: Gc<ast::Expr> =
      self.cx.expr_method_call(
        DUMMY_SP,
        cell,
        self.cx.ident_of("set"),
        vec!(self.cx.expr_binary(DUMMY_SP, ast::BiBitOr, old_masked, new_masked_shifted)));

    let body: P<ast::Block> = self.cx.block(DUMMY_SP, vec!(self.cx.stmt_expr(expr)), None);
    box(GC) ast::Method {
      ident: self.cx.ident_of((String::from_str("set_")+field.name.node).as_slice()),
      attrs: Vec::new(), // TODO: docstring
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

  /// Given an `Expr` of the given register's primitive type, return an `Expr` of the field type
  fn from_primitive(&self, reg: &node::Reg, field: &node::Field, prim: P<ast::Expr>) -> P<ast::Expr> {
    match field.ty.node {
      node::UIntField => prim,
      node::BoolField => self.cx.expr_binary(DUMMY_SP, ast::BiNe, prim, self.expr_int(0)),
      node::EnumField {..} => {
        let from = match reg.ty {
          node::U32Reg => "from_u32",
          node::U16Reg => "from_u16",
          node::U8Reg  => "from_u8",
          _            => fail!("Can't convert group register to primitive type"),
        };
        self.cx.expr_method_call(
          DUMMY_SP,
          self.cx.expr_call_global(
            DUMMY_SP,
            vec!(self.cx.ident_of("core"), self.cx.ident_of("num"), self.cx.ident_of(from)),
            vec!(prim)
          ),
          self.cx.ident_of("unwrap"),
          Vec::new()
        )
      },
    }
  }

  fn emit_field_getter(&self, parent: P<node::RegGroup>, reg: &node::Reg, field: &node::Field)
                       -> P<ast::Method> {
    let ty: P<ast::Ty> = self.cx.ty_path(self.field_type_path(parent, reg, field), None);
    let self_arg: ast::Arg = ast::Arg::new_self(DUMMY_SP, ast::MutImmutable);
    let inputs: Vec<ast::Arg> =
      if field.count.node == 1 {
        vec!(self_arg)
      } else {
        vec!(self_arg) // FIXME
      };
    let decl: P<ast::FnDecl> = self.cx.fn_decl(inputs, ty);

    let (lo,hi) = field.bits.node;
    let mask: P<ast::Expr> = self.expr_int(((1i << (hi-lo+1)) - 1) as i64);
    let value: P<ast::Expr> =
      self.cx.expr_method_call(
        DUMMY_SP,
        self.cx.expr_field_access(DUMMY_SP, self.cx.expr_self(DUMMY_SP), self.cx.ident_of("_value")),
        self.cx.ident_of("get"),
        Vec::new()
      );
    let shifted_masked: P<ast::Expr> =
      self.cx.expr_binary(
        DUMMY_SP,
        ast::BiBitAnd,
        self.cx.expr_binary(DUMMY_SP, ast::BiShr, value, self.expr_int(lo as i64)),
        mask);
    let expr: P<ast::Expr> = self.from_primitive(reg, field, shifted_masked);

    let body: P<ast::Block> = self.cx.block(DUMMY_SP, Vec::new(), Some(expr));
    box(GC) ast::Method {
      ident: self.cx.ident_of(field.name.node.as_slice()),
      attrs: Vec::new(), // TODO: docstring
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

  /// Emit the accessors for a field
  fn emit_field_accessors(&self, parent: P<node::RegGroup>, reg: &node::Reg, field: &node::Field)
                          -> Vec<P<ast::Method>> {
    match field.access {
      node::ReadWrite => vec!(self.emit_field_setter(parent, reg, field),
                              self.emit_field_getter(parent, reg, field)),
      node::WriteOnly => vec!(self.emit_field_setter(parent, reg, field)),
      node::ReadOnly  => vec!(self.emit_field_getter(parent, reg, field)),
    }
  }

  fn emit_register_accessors(&self, parent: P<node::RegGroup>, reg: &node::Reg) -> Vec<P<ast::Item>> {
    let accessors: Vec<Gc<ast::Method>> =
      FromIterator::from_iter(reg.fields.iter().flat_map(|f| self.emit_field_accessors(parent, reg, f).move_iter()));
    let impl_ = ast::ItemImpl(
      no_generics(),
      None,
      self.cx.ty_path(self.cx.path_ident(DUMMY_SP, self.reg_base_type(parent, reg)), None),
      accessors);
    let attrs: Vec<ast::Attribute> = vec!(
      self.list_attribute("allow", vec!("non_snake_case_functions", "dead_code")));
    vec!(self.cx.item(DUMMY_SP, self.cx.ident_of(reg.name.node.as_slice()), attrs, impl_))
  }

  fn emit_group_accessors(&self, group: P<node::RegGroup>) -> Vec<P<ast::Item>> {
    let accessors =
      group.regs.iter().flat_map(|r| self.emit_register_accessors(group, r).move_iter());
    let subgroups = group.groups.values().flat_map(|&g| self.emit_group_accessors(g).move_iter());
    let hi: Vec<P<ast::Item>> = FromIterator::from_iter(subgroups.chain(accessors));
    hi
  }

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
}

pub fn build_ioregs<'a, 'b>(cx: &'a mut ExtCtxt<'b>, groups: HashMap<String, Gc<node::RegGroup>>) -> Builder<'a, 'b> {
  let builder = Builder::new(cx, groups);
  builder
}
