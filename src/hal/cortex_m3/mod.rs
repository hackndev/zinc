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

/*!
Generic routines for ARM Cortex-M3 cores.

This module also provides `isr.rs`, that is not compiled as a part of this
crate. `isr.rs` provides ISR vector table.
*/

pub use super::cortex_common::systick;
pub use super::cortex_common::scb;
pub use super::cortex_common::nvic;
pub use super::cortex_common::mpu;
pub use super::cortex_common::irq;
#[cfg(cfg_multitasking)] pub mod sched;
#[cfg(cfg_multitasking)] pub mod lock;
