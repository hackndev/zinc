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
use core::default;
use core::intrinsics::abort;
use core::option;

#[path="../../util/ioreg.rs"] mod ioreg;
#[path="../../util/wait_for.rs"] mod wait_for;

/// Phase-locked loop clock source.
#[repr(u8)]
pub enum PllClockSource {
  /// Takes base clock from HSI.
  PllSourceHSI = 0,
  /// Takes base clock from HSE.
  PllSourceHSE = 1,
}

/// PLL multiplier: 3, 4, 6, 8, 12, 16, 24, 32, 48
pub type PllMultiplier = u8;

/// PLL divisor: 1, 2, 3, 4
pub type PllDivisor = u8;

/// Multi-speed internal clock divisor.
#[repr(u8)]
pub enum MsiSpeed {
  /// 65.536 kHz
  Msi65   = 0,
  /// 131.072 kHz
  Msi131  = 1,
  /// 262.144 kHz
  Msi262  = 2,
  /// 524.288 kHz
  Msi524  = 3,
  /// 1.048 MHz
  Msi1048 = 4,
  /// 2.097 MHz
  Msi2097 = 5,
  /// 4.194 MHz
  Msi4194 = 6,
}

/// System clock source.
pub enum SystemClockSource {
  /// Multi-speed internal clock,
  SystemClockMSI(MsiSpeed),
  /// High-speed internal oscillator, 16MHz.
  SystemClockHSI,
  /// High-speed external oscillator with configurable frequency.
  SystemClockHSE(u32),
  /// PLL.
  SystemClockPLL(PllClockSource, PllMultiplier, PllDivisor),
}

impl default::Default for SystemClockSource {
  fn default() -> SystemClockSource {
    SystemClockMSI(Msi2097)
  }
}

impl SystemClockSource {
  /// Returns the system clock frequency.
  pub fn frequency(&self) -> u32 {
    match *self {
        SystemClockMSI(Msi65) => 65_536,
        SystemClockMSI(Msi131) => 131_072,
        SystemClockMSI(Msi262) => 262_144,
        SystemClockMSI(Msi524) => 524_288,
        SystemClockMSI(Msi1048) => 1_048_000,
        SystemClockMSI(Msi2097) => 2_097_000,
        SystemClockMSI(Msi4194) => 4_194_000,
        SystemClockHSI => 16_000_000,
        SystemClockHSE(_) => unsafe { abort() }, //TODO(kvark)
        SystemClockPLL(_, _, _) => unsafe { abort() }, //TODO(kvark)
    }
  }
}

#[allow(missing_doc)]
#[repr(u8)]
pub enum McoSource {
  McoClockSystem = 1,
  McoClockHSI = 2,
  McoClockMSI = 3,
  McoClockHSE = 4,
  McoClockPLL = 5,
  McoClockLSI = 6,
  McoClockLSE = 7,
}

/// Microchip clock output configuration.
pub struct McoConfig {
  /// MCO clock source
  source: McoSource,
  /// Log2(divisor) for MCO.
  clock_shift: u8,
}

/// System clock configuration.
pub struct ClockConfig {
  /// System clock source
  pub source : SystemClockSource,
  /// Log2(divisor) for Ahb bus.
  pub ahb_shift : u8,
  /// Log2(divisor) for Apb1 bus.
  pub apb1_shift : u8,
  /// Log2(divisor) for Apb2 bus.
  pub apb2_shift : u8,
  /// Microchip clock output.
  pub mco : option::Option<McoConfig>,
}

impl ClockConfig {
  /// Set this configuration on the hardware.
  pub fn setup(&self) {
    let r = &reg::RCC.cfgr;

    let source_type = match self.source {
      SystemClockMSI(_) => 0,
      SystemClockHSI => 1,
      SystemClockHSE(_) => 2,
      SystemClockPLL(pll_source, mul, div) => {
        r.set_pll_clock_source(pll_source as bool);
        let factor = match mul {
          3 => 0,
          4 => 1,
          6 => 2,
          8 => 3,
          12 => 4,
          16 => 5,
          24 => 6,
          32 => 7,
          48 => 8,
          _ => unsafe { abort() } // not supported
        };
        r.set_pll_mul_factor(factor);
        r.set_pll_output_div(div as u32);
        3
      }
    };

    r.set_system_clock(source_type);
    wait_for!(r.system_clock_status() == source_type);

    if self.ahb_shift > 9 || self.apb1_shift > 4 || self.apb2_shift > 4 {
        unsafe { abort() } // not supported
    }
    r.set_ahb_prescaler(self.ahb_shift as u32);
    r.set_apb1_prescaler(self.apb1_shift as u32);
    r.set_apb2_prescaler(self.apb2_shift as u32);

    match self.mco {
      option::Some(mco) => {
        if mco.clock_shift > 4 {
            unsafe { abort() } // not supported
        }
        r.set_mco(mco.source as u32);
        r.set_mco_prescaler(mco.clock_shift as u32);
      },
      option::None => {
        r.set_mco(0);
      },
    }
  }
}

// TODO(farcaller): this mod is pub as it's being used in peripheral_clock.rs.
//                  This is not the best design solution and a good reason to
//                  split RCC into distinct registers.
#[allow(missing_docs)]
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
      1..0   => system_clock : rw,
      3..2   => system_clock_status: ro,
      7..4   => ahb_prescaler : rw,
      10..8  => apb1_prescaler : rw,
      13..11 => apb2_prescaler : rw,
      16     => pll_clock_source : rw,
      21..18 => pll_mul_factor : rw,
      23..22 => pll_output_div : rw,
      26..24 => mco : rw,   // microcontroller clock output
      30..28 => mco_prescaler : rw,
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
