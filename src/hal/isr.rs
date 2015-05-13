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

//! This file is not part of zinc crate, it is linked separately, alongside the
//! ISRs for the platform.

#![feature(core, asm, lang_items, no_std)]
#![crate_name = "isr"]
#![crate_type = "staticlib"]
#![no_std]

#![allow(missing_docs)]

extern crate core;

#[path="cortex_m3/isr.rs"] pub mod isr_cortex_m3;

#[cfg(mcu_lpc17xx)]
#[path="lpc17xx/isr.rs"] pub mod isr_lpc17xx;

#[cfg(mcu_k20)]
#[path="k20/isr.rs"] pub mod isr_k20;

#[cfg(mcu_tiva_c)]
#[path="tiva_c/isr.rs"] pub mod isr_tiva_c;

#[path="../util/lang_items.rs"] mod lang_items;
