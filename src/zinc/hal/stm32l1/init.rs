// Zinc, the bare metal stack for rust.
// Copyright 2014 Dzmitry "kvark" Malyshau <kvarkus@gmail.com>
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

//! Routines for initialization of STM32L1.
//!
//! This module includes code for setting up the clock, flash, access time and
//! performing initial peripheral configuration.

//use hal::mem_init::init_data;
//use core::intrinsics::abort;

#[path="../../util/ioreg.rs"] mod ioreg;
#[path="../../util/wait_for.rs"] mod wait_for;

// TODO(farcaller): this mod is pub as it's being used in peripheral_clock.rs.
//                  This is not the best design solution and a good reason to
//                  split RCC into distinct registers.
#[allow(missing_doc)]
pub mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(RCC = {
    0x00 => reg32 cr {          // clock control
      31..0 => clock_control : rw,
    },
    0x04 => reg32 icscr {       // internal clock sources calibration
      31..0 => clock_calibration : rw,
    },
    0x08 => reg32 cfgr {        // clock configuration
      31..0 => clock_config : rw,
    },
    0x0C => reg32 cir {         // clock interrupt
      31..0 => clock_interrupt : rw,
    },
    0x10 => reg32 ahbrstr {     // AHB peripheral reset
      31..0 => reset : rw,
    },
    0x14 => reg32 apb2rstr {    // APB2 peripheral reset
      31..0 => reset : rw,
    },
    0x18 => reg32 apb1rstr {    // APB1 peripheral reset
      31..0 => reset : rw,
    },
    0x1C => reg32 ahbenr {      // AHB peripheral clock enable
      31..0 => enable : rw,
    },
    0x20 => reg32 apb2enr {     // APB2 peripheral clock enable
      31..0 => enable : rw,
    },
    0x24 => reg32 apb1enr {     // ABB1 peripheral clock enable
      31..0 => enable : rw,
    },
    0x28 => reg32 ahblpenr {    // AHB peripheral clock enable in low power mode
      31..0 => enable_low_power : rw,
    },
    0x2C => reg32 apb2lpenr {   // APB2 peripheral clock enable in low power mode
      31..0 => enable_low_power : rw,
    },
    0x30 => reg32 apb1lpenr {   // APB1 peripheral clock enable in low power mode
      31..0 => enable_low_power : rw,
    },
    0x34 => reg32 csr {         // control/status
      31..0 => status : rw,
    },
  })

  ioregs!(FLASH = {
    0x00 => reg32 acr {     // access control
      31..0 => access_control : rw,
    },
    0x04 => reg32 pecr {    // program/erase control
      31..0 => program_control : rw,
    },
    0x08 => reg32 pdkeyr {  // power down key
      31..0 => power_down : rw,
    },
    0x0C => reg32 pekeyr {  // program/erase key
      31..0 => program_key : rw,
    },
    0x10 => reg32 prtkeyr { // program memory key
      31..0 => program_memory : rw,
    },
    0x14 => reg32 optkeyr { // option byte key
      31..0 => option_byte : rw,
    },
    0x18 => reg32 sr {      // status register
      31..0 => status : rw,
    },
    0x1C => reg32 obr {     // option byte
      31..0 => option : rw,
    },
    0x20 => reg32 wrpr {    // write protection
      31..0 => protect : rw,
    },
    0x28 => reg32 wrpr1 {   // write protection register 1
      31..0 => protect : rw,
    },
    0x2C => reg32 wrpr2 {   // write protection register 2
      31..0 => protect : rw,
    },
  })

  ioregs!(PWR = {
    0x0 => reg32 cr {   // power control
      31..0 => control : rw,
    },
    0x4 => reg32 csr {  // power control/status
      31..0 => status : rw,
    },
  })

  extern {
    #[link_name="stm32l1_iomem_RCC"] pub static RCC: RCC;
    #[link_name="stm32l1_iomem_FLASH"] pub static FLASH: FLASH;
    #[link_name="stm32l1_iomem_PWR"] pub static PWR: PWR;
  }
}
