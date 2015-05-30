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

#![feature(asm, lang_items, plugin, core)]
#![allow(improper_ctypes)]
#![deny(missing_docs)]
#![no_std]

/*!
Zinc is an embedded stack for rust.

Zinc provides a complete embedded stack for application development in rust. It
is provided in a form of library, compiled for a specific MCU, that can be
linked into user's own applications.

### Supported architectures

ARM is the only architecture, supported at the moment. Zinc can be compiled for
"native" architecture as well, which should be useful only for testing the code,
though.

### Supported ARM MCUs

Two MCUs are supported at the moment, specifically

 * NXP LPC1768
 * ST STM32F407
 * ST STM32L152RCT6
 * Freescale MK20DX32
 * TI TM4C123GXL

The code is generic enough to support other MCUs in the same family (LPC17xx and
STM32F403/407).
*/
#![plugin(ioreg)]
#![feature(no_std)]

#[macro_use] extern crate core;
#[cfg(not(test))] extern crate rlibc;

#[macro_use] #[no_link] extern crate ioreg;

#[cfg(test)] extern crate std;

pub mod drivers;
pub mod hal;
pub mod util;
pub mod os;

/// Export a subset of `core` as `std.
///
/// This is exported as a temporary convenience as `rustc` still looks
/// for some items in the `std` namespace.
///
/// TODO(farcaller): clean up when fixed.
#[cfg(not(test))]
pub mod std {
  pub use core::cmp;  // used for #[derive(Eq)] until fixed in rust.
  pub use core::option;
  pub use core::num;
  pub use core::marker;
}
