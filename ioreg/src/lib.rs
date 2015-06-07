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

/*!
## I/O register interface

On most embedded platforms interaction with hardware peripherals
occurs through memory-mapped registers. This crate provides a syntax
extension for `rustc` to generate convenient, type-safe register
interfaces from a minimal definition.

### Concepts

A *register* is group of bits (typically a word, although on some
platforms smaller). By manipulating the bits of a register one can
affect the state of the associated peripheral.

### Example register definition

Let's consider a register block for a simple UART-like peripheral. The
documentation for the peripheral will likely have a table defining the
interface resembling the following,

```notrust
offset       name         description
───────      ────────     ──────────────────────────
0x0          CR           Configuration register
  bit 0      RXE          Receive enable
  bit 1      TXE          Transmit enable
  bit 2      RXIE         Recieve interrupt enable
  bit 3      TXIE         Transmit interrupt enable
  bit 12:4   BR           Baudrate
  bit 16:14  PARITY       Parity
                            0x0     No parity
                            0x1     Reserved
                            0x2     Even parity
                            0x3     Odd parity

0x4          SR           Status register
  bit 0      RXNE         Receive data register not empty flag (read-only)
  bit 1      TXE          Transmit data register not empty flag (read-only)
  bit 2      FE           Framing error flag (set to clear)

0x8          DR           Data register
  bits 7:0   D            Read returns received data
                          Write transmits data
```

The syntax extension is invoked through through the `ioregs!` macro. A
register definition for the above peripheral might look like this,

```
ioregs!(UART = {
    0x0    => reg32 cr {
        0      => rxe,
        1      => txe,
        2      => rxie,
        3      => txie,
        4..12  => br,
        14..16 => parity {
          0x0  => NoParity,
          0x2  => EvenParity,
          0x3  => OddParity,
        }
    }

    0x4    => reg32 sr {
        0      => rxne: ro,
        1      => txe: ro,
        2      => fe: set_to_clear,
    }

    0x8    => reg32 dr {
        0..7   => d
    }
})
```

Here we've defined a register block called `UART` consisting of a
three registers: `cr`, `sr`, and `dr`. Each register definition
consists of an offset from the beginning of the register block width,
a register type giving the width of the register (`reg32` in this
case), a name, and a list of fields.

The `cr` register has four boolean flags, an integer
field `br`, and a field `parity` with four possible values
(`NoParity`, `EvenParity`, and `OddParity`). Each field is defined by
a bit or bit range, a name, some optional modifiers (e.g. `ro` in the
case of `rxne`), and an optional list of values.

This register definition will produce a variety of types, along with
associated accessor methods for convenient, safe manipulation of the
described registers. In the process of generating these, `ioregs!`
will perform a variety of sanity checks (e.g. ensuring that registers
and bitfields are free of overlap).

#### Documenting register definitions

It is highly recommended that register definitions include
docstrings. Registers, fields, and `enum` values can all be annotated
with docstrings with the typical Rust doc comment syntax. Both outer
(`/// comment`) and inner (`//! comment`) comments are accepted. Inner
comments apply to the item to which the current block belongs whereas
outer comments apply to the item that follows. In addition,
trailing comments are supported with the `//=` syntax. These apply
to the preceding item, allowing definitions and associated comments
to inhabit the same line. Multiple successive comments of the same
type will be concatenated together into a single doc comment.

For instance, we might document the above example as follows,

```
ioregs!(UART = {
    /// Control register
    /// Here is some discussion of the function of the `cr` register.
    0x0    => reg32 cr {
        0      => rxe,         //= Receive enable
        1      => txe,         //= Transmit enable
        2      => rxie,        //= Receive interrupt enable
        3      => txie,        //= Transmit interrupt enable
        4..12  => br,          //= Baud rate
        14..16 => parity {     //! Parity selection
          0x0  => NoParity,    //= No parity
          0x2  => EvenParity,  //= Even parity
          0x3  => OddParity,   //= Odd parity
        }
    }

    ...
})
```

#### Nesting register blocks

In addition to primitive register types (e.g. `reg32`), one can also
nest groups of logically related registers. For instance, in the case
of a DMA peripheral it is common that the same block of registers will
be replicated, one for each DMA channel. This can be accomplished with
`ioregs!` as follows,

```
ioregs!(DMA = {
    0x0    => reg32 cr { ... }
    0x10   => group channel[4] {
        0x0    => reg32 cr { ... }
        0x4    => reg32 sr { ... }
    }
    0x30   => reg32 sr { ... }
})
```

This will produce the following layout in memory,

```notrust
address        register
────────       ──────────────
0x0            cr
0x10           channel[0].cr
0x14           channel[0].sr
0x18           channel[1].cr
0x1c           channel[1].sr
0x20           channel[2].cr
0x24           channel[2].sr
0x28           channel[3].cr
0x2c           channel[3].sr
0x30           sr
```

### What is produced

The `ioregs!` extension produces a variety of types and methods for
each register and field. Let's start by examining the top-level types
representing the structure of the interface.

```
pub enum UART_cr_parity {
    NoParity = 0, EvenParity = 2, OddParity = 3,
}

pub struct UART_cr { ... }
pub struct UART_sr { ... }
pub struct UART_dr { ... }

pub struct UART {
    pub cr: UART_cr,
    pub sr: UART_sr,
    pub dr: UART_dr,
}
```

The `UART` struct is the the "entry-point" into the interface and is
ultimately what will be instantiated to represent the peripheral's
register window, typically as a `static extern` item,

```
extern { pub static UART: UART; }
```

The register structs (`UART_cr`, `UART_sr`, and `UART_dr`)
have no user visible members but expose a variety of methods. Let's
look at `cr` in particular,

```
impl UART_cr {
    pub fn get(&self) -> UART_cr_Get { ... }

    pub fn set_rxe(&self, new_value: bool) -> UART_cr_Update { ... }
    pub fn rxe(&self) -> bool { ... }

    // similar methods for `txe`, `rxie`, `txie`

    pub fn set_br(&self, new_value: u32) -> UART_cr_Update { ... }
    pub fn br(&self) -> u32 { ... }

    pub fn set_parity(&self, new_value: UART_cr_parity) -> UART_cr_Update { ... }
    pub fn parity(&self) -> UART_cr_parity { ... }
}
```

Here we see each field has a corresponding "get" function (e.g. `rxe`,
`br`, and `parity`) as well as a "set" function. Note that the set
function returns a `UART_cr_Update` object. This object mirrors the
setter methods of `UART_cr`, collecting multiple field updates within
a register, performing them on destruction with the `Drop` trait,

```
pub struct UART_cr_Update { ... }
impl Drop for UART_cr_Update { ... }

impl UART_cr_Update {
    pub fn set_rxe<'a>(&'a mut self, new_value: bool) -> &'a mut UART_cr_Update { ... }
    pub fn set_txe<'a>(&'a mut self, new_value: bool) -> &'a mut UART_cr_Update { ... }
    ...
}
```

As the set methods return references to `self` they can be easily
chained together. For instance, we can update the `rxe` and `txe`
fields of the `cr` register atomically,

```
UART.cr.set_rxe(true).set_txe(false);
```

In addition to get and set methods, `UART_cr` also implements a `get`
method which returns a `UART_cr_Get` object mirroring the get methods
of `UART_cr`. This object captures the state of the register allowing
field values to be later atomically queried,

```
let cr: UART_cr_Get = UART.cr.get();
format!("txe={}, rxe={}, br={}", cr.txe(), cr.rxe(), cr.br())
```

In the case of read-only (resp. write-only) fields the set (resp. get)
method is omitted. In the case of `set_to_clear` fields a `clear`
method is instead produced in place of `set`. For instance, in the
case of the `sr` register's `fe` flag,

```
pub fn fe(self: &UART_sr_Getter) -> bool { ... }
pub fn clear_fe(self: &UART_sr_Update) -> UART_sr_Update { ... }
```

### Informal grammar

In the below discussion `THING, ...` will denote a list of one or more
`THING`s. The `THING`s must be comma separated except when ending
with a brace-enclosed block. Optional elements are enclosed in `⟦...⟧`
brackets.

The `ioregs!` macro expects a definition of the form,

```
ioregs!(IDENT = { REG, ... })
```

Where a `REG` is either a register group,

```notrust
OFFSET => group IDENT⟦[COUNT]⟧ { REG, ... }
```

or a primitive register,

```notrust
OFFSET => TYPE IDENT⟦[COUNT]⟧ { FIELD, ... }
```

`COUNT` is an integer count and a register `TYPE` is one of `reg8` (a
one byte wide register), `reg16` (two bytes wide), or `reg32` (four
bytes wide).

A field is given by

```notrust
BITS => IDENT⟦[COUNT]⟧ ⟦: MODIFIER⟧ ⟦{ VALUE, ... }⟧
```

where `BITS` is either an inclusive range of integers (`N..M`) or a
single integer (shorthand for `N..N`). If a list of values is given
the field is of an enumerated type. Otherwise single bit fields are
of type `bool` and wider fields unsigned integers (in particular, of
the same width as the containing register).

A `MODIFIER` is one of `rw` (read/write), `ro` (read-only), `wo`
(write-only), or `set_to_clear` (a flag which can be cleared by
setting to one).

A `VALUE` is given by,

```notrust
N => NAME
```

*/

#![feature(quote, plugin_registrar, rustc_private, collections, core)]
#![feature(convert)]
#![feature(plugin)]

#![plugin(syntaxext_lint)]

extern crate rustc;
extern crate syntax;
extern crate serialize;

use rustc::plugin::Registry;
use syntax::ast;
use syntax::ptr::P;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::util::small_vector::SmallVector;

pub mod node;
pub mod parser;
pub mod builder;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("ioregs", macro_ioregs);
}

pub fn macro_ioregs(cx: &mut ExtCtxt, _: Span, tts: &[ast::TokenTree])
                    -> Box<MacResult+'static> {
  match parser::Parser::new(cx, tts).parse_ioregs() {
    Some(group) => {
      let mut builder = builder::Builder::new();
      let items = builder.emit_items(cx, group);
      MacItems::new(items)
    },
    None => {
      panic!("Parsing failed");
    }
  }
}

pub struct MacItems {
  items: Vec<P<ast::Item>>
}

impl MacItems {
  pub fn new(items: Vec<P<ast::Item>>) -> Box<MacResult+'static> {
    Box::new(MacItems { items: items })
  }
}

impl MacResult for MacItems {
  fn make_items(self: Box<MacItems>) -> Option<SmallVector<P<ast::Item>>> {
    Some(SmallVector::many(self.items.clone()))
  }
}
