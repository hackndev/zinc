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

#![feature(globs, macro_rules, asm, phase, unsafe_destructor, lang_items)]
#![crate_name="zinc"]
#![crate_type="rlib"]
#![allow(ctypes)]
#![deny(missing_doc)]
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

The code is generic enough to support other MCUs in the same family (LPC17xx and
STM32F403/407).
*/

extern crate core;
extern crate rlibc;

#[cfg(test)] #[phase(plugin,link)] extern crate std;
#[cfg(test)] extern crate native;
#[phase(plugin)] extern crate ioreg;

pub mod drivers;
pub mod hal;
pub mod lib;
pub mod os;

// TODO(farcaller): clean up when fixed.
#[cfg(not(test))]
mod std {
  pub use core::cmp;  // used for #[deriving(Eq)] until fixed in rust.
  pub use core::option;
  pub use core::num;
}
