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
HAL provides abstractions for specific MCU hardware.

Each peripheral in `hal` has a `xxxConf` struct that can be defined statically,
and each such struct has a `setup()` method that configures the hardware
(returning the object to interact with it where applicable).
*/

#[cfg(feature = "mcu_lpc11xx")] pub mod lpc11xx;
#[cfg(feature = "mcu_lpc17xx")] pub mod lpc17xx;
#[cfg(feature = "mcu_stm32f4")] pub mod stm32f4;
#[cfg(feature = "mcu_stm32l1")] pub mod stm32l1;
#[cfg(feature = "mcu_k20")] pub mod k20;
#[cfg(feature = "mcu_tiva_c")] pub mod tiva_c;

#[cfg(any(feature = "cpu_cortex-m0",
          feature = "cpu_cortex-m3",
          feature = "cpu_cortex-m4"))]
mod cortex_common;
#[cfg(feature = "cpu_cortex-m3")]
pub mod cortex_m3;
#[cfg(feature = "cpu_cortex-m4")]
pub mod cortex_m4;

pub mod mem_init;
pub mod pin;
pub mod pwm;
pub mod spi;
pub mod stack;
pub mod timer;
pub mod uart;

#[cfg(target_os = "none")]
pub mod isr;
