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
HAL provides abstactions over different peripherals found in MCUs (vs drivers,
that provide support for hardware outside of MCU).

HAL is mostly implemented as `pub use` for the relevant modules in MCU-specific
directories. Each peripheral has a Conf struct, that can be defined statucally,
and each such struct has a `setup()` method that configures the hardware,
returning the object to interact with it where applicable.
*/

mod mem_init;

#[cfg(mcu_lpc17xx)]
#[path="lpc17xx/mod.rs"] pub mod lpc17xx;

pub mod pin;
pub mod gpio;
pub mod init;
pub mod timer;
pub mod uart;
pub mod spi;
