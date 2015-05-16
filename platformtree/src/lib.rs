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

//! Platform tree operations crate

#![feature(quote, rustc_private, collections, alloc, hash, convert)]

// extern crate regex;
extern crate syntax;
#[cfg(test)] extern crate hamcrest;

pub mod builder;
pub mod node;
pub mod parser;

#[path="../../src/hal/lpc17xx/platformtree.rs"] mod lpc17xx_pt;
// #[path="../zinc/hal/tiva_c/platformtree.rs"] mod tiva_c_pt;
#[path="../../src/drivers/drivers_pt.rs"] mod drivers_pt;

#[cfg(test)] mod test_helpers;
#[cfg(test)] mod parser_test;
