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

//! Routines for initialization of STM32F1.
//!
//! This module includes code for setting up the clock, flash, access time and
//! performing initial peripheral configuration.

//use hal::mem_init::init_data;
use core::default;

use self::SystemClockSource::*;
use self::PllClockSource::*;
use self::PllHsePrediv::*;
use self::PllUsbDiv::*;
use self::PllMult::*;
use self::ClockAhbPrescaler::*;
use self::ClockApbPrescaler::*;
use self::FlashLatency::*;
use self::McoSource::*;

#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Phase-locked loop clock source.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PllClockSource {
  /// Takes base clock from HSI/2.
  PllSourceHSIDiv2,
  /// Takes base clock from HSE.
  PllSourceHSE(u32),
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PllMult {
  /*  0 = 0b0000 */ PllMul2,
  /*  1 = 0b0001 */ PllMul3,
  /*  2 = 0b0010 */ PllMul4,
  /*  3 = 0b0011 */ PllMul5,
  /*  4 = 0b0100 */ PllMul6,
  /*  5 = 0b0101 */ PllMul7,
  /*  6 = 0b0110 */ PllMul8,
  /*  7 = 0b0111 */ PllMul9,
  /*  8 = 0b1000 */ PllMul10,
  /*  9 = 0b1001 */ PllMul11,
  /* 10 = 0b1010 */ PllMul12,
  /* 11 = 0b1011 */ PllMul13,
  /* 12 = 0b1100 */ PllMul14,
  /* 13 = 0b1101 */ PllMul15,
  /* 14 = 0b1110 */ PllMul16,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PllHsePrediv {
    PllHsePrediv1,
    PllHsePrediv2,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PllUsbDiv {
    PllUsbDiv1,
    PllUsbDiv1p5,
}

#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct PllConf {
    pub source: PllClockSource,
    pub mult: PllMult,
    pub hse_prediv: PllHsePrediv,
    pub usb_prescaler: PllUsbDiv,
}

/// System clock source.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum SystemClockSource {
  /// High-speed internal oscillator, 8MHz.
  SystemClockHSI,
  /// High-speed external oscillator with configurable frequency.
  SystemClockHSE(u32),
  /// PLL.
  SystemClockPLL(PllConf),
}

impl default::Default for SystemClockSource {
  fn default() -> SystemClockSource {
    SystemClockHSI
  }
}

impl SystemClockSource {
  /// Returns the system clock frequency.
  pub fn frequency(&self) -> u32 {
    match *self {
        SystemClockHSI => 8_000_000,
        SystemClockHSE(frequency) => frequency,
        SystemClockPLL(pll_conf) => {
            let pll_in = match pll_conf.source {
                PllSourceHSIDiv2 => 4_000_000,
                PllSourceHSE(frequency) => match pll_conf.hse_prediv {
                    PllHsePrediv1 => frequency,
                    PllHsePrediv2 => frequency >> 1,
                },
            };
            let mul = pll_conf.mult as u32 + 2;
            pll_in * mul
        }
    }
  }
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum McoSource {
  McoClockNone,
  McoClockSys,
  McoClockHSI,
  McoClockHSE,
  McoClockPLL,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ClockAhbPrescaler {
    AhbDivNone,
    AhbDiv2,
    AhbDiv4,
    AhbDiv8,
    AhbDiv16,
    AhbDiv64,
    AhbDiv128,
    AhbDiv256,
    AhbDiv512,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ClockApbPrescaler {
    ApbDivNone,
    ApbDiv2,
    ApbDiv4,
    ApbDiv8,
    ApbDiv16,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum FlashLatency {
    FlashLatency0,
    FlashLatency1,
    FlashLatency2,
}

/// System clock configuration.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct ClockConfig {
  pub source : SystemClockSource,
  pub ahb_prescaler : ClockAhbPrescaler,
  pub apb1_prescaler : ClockApbPrescaler,
  pub apb2_prescaler : ClockApbPrescaler,
  pub flash_latency : FlashLatency,
  pub mco : McoSource,
}

impl default::Default for ClockConfig {
  fn default() -> ClockConfig {
    ClockConfig {
      source: default::Default::default(),
      ahb_prescaler : AhbDivNone,
      apb1_prescaler : ApbDivNone,
      apb2_prescaler : ApbDivNone,
      flash_latency : FlashLatency0,
      mco: McoClockNone,
    }
  }
}

impl ClockConfig {
  /// Return the default clock configuration that hardware go to after reset.
  pub fn new_default() -> ClockConfig {
    default::Default::default()
  }

  /// Set this configuration on the hardware.
  pub fn setup(&self) {
    let rcc = &reg::RCC;
    let flash = &reg::FLASH;

    let source_type = match self.source {
      SystemClockHSI => {
        rcc.cr.set_hsi_on(true);
        wait_for!(rcc.cr.hsi_ready());
        0b00  // system_clock = HSI
      },
      SystemClockHSE(_) => {
        rcc.cr.set_hse_on(true);
        wait_for!(rcc.cr.hse_ready());
        0b01  // system_clock = HSE
      },
      SystemClockPLL(pll_conf) => {
        match rcc.cfgr.system_clock() {
            // if PLL is current clock source, temporarly switch it to HSI
            // because otherwise we cannot modify it
            // TODO(blazewicz): check if HSI is on
            0b10 => {
                rcc.cfgr.set_system_clock(0b00);
                wait_for!(rcc.cfgr.system_clock_status() == 0b00);
            },
            _ => ()
        };

        // disable PLL
        rcc.cr.set_pll_on(false);
        wait_for!(!rcc.cr.pll_ready());

        // set pll clock source
        let pll_clock_source = match pll_conf.source {
            PllSourceHSIDiv2 => {
                rcc.cr.set_hsi_on(true);
                wait_for!(rcc.cr.hsi_ready());
                false // pll_clock_source = HSI divided by 2
            },
            PllSourceHSE(_)  => {
                rcc.cr.set_hse_on(true);

                // set HSE divider for PLL entry
                let pll_hse_divider = match pll_conf.hse_prediv {
                    PllHsePrediv1 => false,
                    PllHsePrediv2 => true,
                };
                rcc.cfgr.set_pll_hse_divider(pll_hse_divider);

                wait_for!(rcc.cr.hse_ready());
                true // pll_clock_source = HSE
            },
        };
        rcc.cfgr.set_pll_clock_source(pll_clock_source);

        // set pll multiplication factor
        rcc.cfgr.set_pll_mul_factor(pll_conf.mult as u32);

        // set USB prescaler (max 48 MHz)
        let usb_prescaler = match pll_conf.usb_prescaler {
            PllUsbDiv1p5 => false,
            PllUsbDiv1   => true,
        };
        rcc.cfgr.set_usb_prescaler(usb_prescaler);

        // enable PLL
        rcc.cr.set_pll_on(true);
        wait_for!(rcc.cr.pll_ready());

        0b10 // system_clock = PLL
       }
    };

    /* TODO(blazewicz): configuring flash latency is straightforward
     * and could be done automatically:
     *       0 < SYSCLK <= 24 MHz => 0
     *  24 MHz < SYSCLK <= 48 MHz => 1
     *  48 MHz < SYSCLK <= 72 MHz => 2
     */
    flash.acr.set_latency(self.flash_latency as u32);

    rcc.cfgr.set_system_clock(source_type);
    wait_for!(rcc.cfgr.system_clock_status() == source_type);

    let ahb_select = match self.ahb_prescaler {
        AhbDivNone => 0b0000u32,
        AhbDiv2    => 0b1000u32,
        AhbDiv4    => 0b1001u32,
        AhbDiv8    => 0b1010u32,
        AhbDiv16   => 0b1011u32,
        //NOTE(blazewicz): there is no /32
        AhbDiv64   => 0b1100u32,
        AhbDiv128  => 0b1101u32,
        AhbDiv256  => 0b1110u32,
        AhbDiv512  => 0b1111u32,
    };
    rcc.cfgr.set_ahb_prescaler(ahb_select);

    // (max 36 MHz)
    let apb1_select = match self.apb1_prescaler {
        ApbDivNone => 0b000u32,
        ApbDiv2    => 0b100u32,
        ApbDiv4    => 0b101u32,
        ApbDiv8    => 0b110u32,
        ApbDiv16   => 0b111u32,
    };
    rcc.cfgr.set_apb1_prescaler(apb1_select);

    let apb2_select = match self.apb2_prescaler {
        ApbDivNone => 0b000u32,
        ApbDiv2    => 0b100u32,
        ApbDiv4    => 0b101u32,
        ApbDiv8    => 0b110u32,
        ApbDiv16   => 0b111u32,
    };
    rcc.cfgr.set_apb2_prescaler(apb2_select);

    let mco_select = match self.mco {
        McoClockNone => 0b000u32,
        McoClockSys  => 0b100u32,
        McoClockHSI  => 0b101u32,
        McoClockHSE  => 0b110u32,
        McoClockPLL  => 0b111u32,
    };
    rcc.cfgr.set_mco(mco_select);
  }

  /// Returns AHB clock frequency
  pub fn get_ahb_frequency(&self) -> u32 {
      let shift = match self.ahb_prescaler {
          AhbDivNone => 0,
          AhbDiv2    => 1,
          AhbDiv4    => 2,
          AhbDiv8    => 3,
          AhbDiv16   => 4,
          AhbDiv64   => 6,
          AhbDiv128  => 7,
          AhbDiv256  => 8,
          AhbDiv512  => 9,
      };
      self.source.frequency() >> shift
  }

  /// Returns APB1 clock frequency
  pub fn get_apb1_frequency(&self) -> u32 {
      let shift = match self.apb1_prescaler {
          ApbDivNone => 0,
          ApbDiv2    => 1,
          ApbDiv4    => 2,
          ApbDiv8    => 3,
          ApbDiv16   => 4,
      };
      self.get_ahb_frequency() >> shift
  }

  /// Returns APB2 clock frequency
  pub fn get_apb2_frequency(&self) -> u32 {
      let shift = match self.apb2_prescaler {
          ApbDivNone => 0,
          ApbDiv2    => 1,
          ApbDiv4    => 2,
          ApbDiv8    => 3,
          ApbDiv16   => 4,
      };
      self.get_ahb_frequency() >> shift
  }
}

// TODO(farcaller): this mod is pub as it's being used in peripheral_clock.rs.
//                  This is not the best design solution and a good reason to
//                  split RCC into distinct registers.
#[allow(missing_docs)]
pub mod reg {
  use volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(RCC = {
    0x00 => reg32 cr {          // clock control
      0 => hsi_on : rw,
      1 => hsi_ready : ro,
      7..3 => hsi_trim : rw,
      15..8 => hsi_cal : rw,
      16 => hse_on : rw,
      17 => hse_ready : ro,
      18 => hse_bypass : rw,
      19 => css_on : rw,
      24 => pll_on : rw,
      25 => pll_ready : ro,
    },
    0x04 => reg32 cfgr {        // clock configuration
      1..0   => system_clock : rw,
      3..2   => system_clock_status: ro,
      7..4   => ahb_prescaler : rw,
      10..8  => apb1_prescaler : rw,
      13..11 => apb2_prescaler : rw,
      15..14 => adc_prescaler : rw,
      16     => pll_clock_source : rw,
      17     => pll_hse_divider : rw,
      21..18 => pll_mul_factor : rw,
      22     => usb_prescaler : rw,
      26..24 => mco : rw,
    },
    0x08 => reg32 cir {         // clock interrupt
      31..0 => clock_interrupt : rw,
    },
    0x0C => reg32 apb2rstr {    // APB2 peripheral reset
      31..0 => reset : rw,
    },
    0x10 => reg32 apb1rstr {    // APB1 peripheral reset
      31..0 => reset : rw,
    },
    0x14 => reg32 ahbenr {      // AHB peripheral clock enable
      31..0 => enable : rw,
    },
    0x18 => reg32 apb2enr {     // APB2 peripheral clock enable
      31..0 => enable : rw,
    },
    0x1C => reg32 apb1enr {     // ABB1 peripheral clock enable
      31..0 => enable : rw,
    },
    0x20 => reg32 bdcr {
      0    => lse_on : rw,
      1    => lse_ready : ro,
      2    => lse_bypass : rw,
      9..8 => rtc_source : rw,
      15   => rtc_on : rw,
      16   => backup_reset : rw,
    }
    0x24 => reg32 csr {         // control/status
      0 => lsi_on : rw, // internal low speed oscillator
      1 => lsi_ready : ro,
      24 => remove_reset : rw,
      26 => pin_reset : rw,
      27 => pop_pdr_reset : rw,
      28 => software_reset : rw,
      29 => independent_watchdog_reset : rw,
      30 => window_watchdog_reset : rw,
      31 => low_power_reset : rw,
    },
  });

  ioregs!(FLASH = {
    0x00 => reg32 acr {     // access control
      2..0 => latency : rw,
      3    => flash_half_cycle_on : rw,
      4    => prefetch_buf_on : rw,
      5    => prefetch_buf_status : ro,
    },
  });

  ioregs!(PWR = {
    0x0 => reg32 cr {   // power control
      31..0 => control : rw,
    },
    0x4 => reg32 csr {  // power control/status
      31..0 => status : rw,
    },
  });

  extern {
    #[link_name="stm32f1_iomem_RCC"] pub static RCC: RCC;
    #[link_name="stm32f1_iomem_FLASH"] pub static FLASH: FLASH;
    #[link_name="stm32f1_iomem_PWR"] pub static PWR: PWR;
  }
}
