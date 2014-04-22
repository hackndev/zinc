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

#![feature(globs, macro_rules, asm)]
#![crate_id="zinc"]
#![crate_type="rlib"]
#![allow(ctypes)]
#![no_std]

/*!
Zinc is an embedded stack for rust.

Zinc provides a complete embedded stack for application development in rust. It
is provided in a form of library, compiled for a specific MCU, that can be
linked into user's own applications.

Zinc supports only ARM MCUs at the moment, specifically:

 * NXP LPC1768
 * ST STM32F407

The library includes all available drivers, but rust has a damn good LTO, so you
don't need to worry about resulting binary size.
*/

extern crate core;

/// hardware abstractions. Includes implementations for things found inside MCUs.
pub mod hal;

/// (deprecated) public interfaces.
pub mod interfaces;

/// drivers for various external peripherals, like TFT LCDs and radios.
pub mod drivers;

/// support code not included with rust-core.
pub mod lib;

/// Default configurations for various boards.
pub mod boards;
