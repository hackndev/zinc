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

//! Routines for initialization of STM32L1.
//!
//! This module includes code for setting up the clock, flash, access time and
//! performing initial peripheral configuration.

// THIS WHOLE THING IS COPIED FROM stm32f4, NEEDS TO BE PORTED PROPERLY
// SO FAR ONLY THE REGISTER SUBMODULE IS PORTED

use hal::mem_init::init_data;
use core::intrinsics::abort;

#[path="../../util/ioreg.rs"] mod ioreg;
#[path="../../util/wait_for.rs"] mod wait_for;

/// System clock source.
pub enum SystemClockSource {
  /// High-speed internal oscillator, 16MHz.
  SystemClockHSI,
  /// High-speed external oscillator with configurable frequency.
  SystemClockHSE(u32),
  /// PLL.
  SystemClockPLL(PLLConf),
}

/// PLL clock source. Applies to both PLL and PLLI2S.
pub enum PLLClockSource {
  /// High-speed internal oscillator, 16MHz.
  PLLClockHSI,
  /// High-speed external oscillator with configurable frequency.
  PLLClockHSE(u32),
}

/// PLL configuration options.
///
/// Frequency is calculated as
///
/// ```
/// Fvco = F_pll_in * n / m
/// F_pll_out = Fvco / p
/// F_pll_usb = Fvco / q
/// ```
pub struct PLLConf {
  /// Clock source.
  pub source: PLLClockSource,
  /// Division factor for the main PLL and PLLI2S input clock.
  pub m: u8,
  /// Multiplication factor for the main PLL and PLLI2S input clock.
  pub n: u16,
  /// Main PLL division factor for main system clock.
  pub p: u8,
  /// Main PLL division factor for USB, SDIO and RNG.
  pub q: u8,
}

/// MCU clock configuration.
pub struct ClockConf {
  /// Clocking source.
  pub source: SystemClockSource,
}

/// MCU configuration.
pub struct SysConf {
  /// Clock configuration.
  pub clock: ClockConf,
}

// TODO(farcaller): move to peripheral_clock?
static mut SystemClock: u32 = 0;

/// Returns system clock frequency according to configuration.
#[inline(always)]
pub fn system_clock() -> u32 {
  unsafe { SystemClock }
}

// TODO(farcaller): move to peripheral_clock?
static mut APBLowClock: u32 = 0;

/// Returns system clock frequency according to configuration.
#[inline(always)]
pub fn apb_low_clock() -> u32 {
  unsafe { APBLowClock }
}

impl SysConf {
  /// Performs the MCU initialization.
  pub fn setup(&self) {
    init_data();
    self.clock.setup();
  }
}

impl ClockConf {
  fn setup(&self) {
    match self.source {
      SystemClockHSI => {
        // HSI is default boot mode, do nothing
        unsafe {
          SystemClock = 16_000_000;
          APBLowClock = 16_000_000;  // no divisor
        };
      },
      SystemClockHSE(freq) => {
        // Switch to HSE
        if freq > 30_000_000 {
          // this code doesn't support wait states configuration for HSE
          unsafe { abort() };
        } else {
          self.enable_hse();
          self.set_system_clock(reg::SystemClockHSE);
          unsafe {
            SystemClock = freq;
            APBLowClock = freq;  // no divisor
          };
        }
      },
      SystemClockPLL(pll_conf) => {
        // Init and switch to pll
        match pll_conf.source {
          PLLClockHSE(_) => self.enable_hse(),
          _ => (),
        }
        let sysfreq: u32 = match pll_conf.source {
          PLLClockHSE(freq) => freq,
          PLLClockHSI       => 16_000_000,
        } as u32 * (pll_conf.n as u32 / pll_conf.m as u32) / pll_conf.p as u32;
        // system_stm32f4xx.c enables PWR and sets VOS to 1 here, but VOS
        // defaults to 1 so I see no real reason to do that.
        // peripheral_clock::PWRClock.enable();
        // reg::PWR.set_CR(reg::PWR.CR() | 0x0000C000);

        // TODO(farcaller): this should be configureable via ClockConf
        let apb_low_divisor = 4;
        self.set_clock_divisors(1, apb_low_divisor, 2);
        pll_conf.setup();
        // TODO(farcaller): this doesn't really belong here.
        self.setup_flash(sysfreq);
        self.set_system_clock(reg::SystemClockPLL);
        unsafe {
          SystemClock = sysfreq;
          APBLowClock = sysfreq / apb_low_divisor as u32;
        };
      },
    }
  }

  fn setup_flash(&self, freq: u32) {
    reg::FLASH.set_ACR(
      (1 << 8)  |  // enable prefetch
      (1 << 9)  |  // enable instruction cache
      (1 << 10) |  // enable data cache
      match freq/1_000_000 {  // wait states are calculated for 2.7-3.6V range
        0...30    => 0,
        31...60   => 1,
        61...90   => 2,
        90...120  => 3,
        121...150 => 4,
        151...168 => 5,
        _        => unsafe { abort() },
      }
    );
  }

  fn enable_hse(&self) {
    let val = reg::RCC.CR();
    let hse_on_bit: u32 = 1 << 16;
    let hse_ready_bit: u32 = 1 << 17;
    reg::RCC.set_CR(val | hse_on_bit);

    wait_for!(reg::RCC.CR() & hse_ready_bit == hse_ready_bit);
  }

  fn set_system_clock(&self, clock: reg::SystemClockSwitch) {
    let val = reg::RCC.CFGR();
    let bits: u32 = clock as u32;
    let mask: u32 = !0b1111;

    reg::RCC.set_CFGR((val & mask) | bits);
  }

  fn set_clock_divisors(&self, ahb: u16, apb_low_speed: u8, apb_hi_speed: u8) {
    let val = reg::RCC.CFGR();
    let mask: u32 = !0b111_111_00_1111_0000;
    let bits: u32 = (match ahb {
      1   => 0b0000,
      2   => 0b1000,
      4   => 0b1001,
      8   => 0b1010,
      16  => 0b1011,
      64  => 0b1100,
      128 => 0b1101,
      256 => 0b1110,
      512 => 0b1111,
      _   => unsafe { abort() },
    } << 4) | (match apb_low_speed {
      1   => 0b000,
      2   => 0b100,
      4   => 0b101,
      8   => 0b110,
      16  => 0b111,
      _   => unsafe { abort() },
    } << 10) | (match apb_hi_speed {
      1   => 0b000,
      2   => 0b100,
      4   => 0b101,
      8   => 0b110,
      16  => 0b111,
      _   => unsafe { abort() },
    } << 13);

    reg::RCC.set_CFGR((val & mask) | bits);
  }
}

impl PLLConf {
  fn setup(&self) {
    // TODO
    unsafe { abort(); }
  }

  fn enable_pll(&self) {
    let val = reg::RCC.CR();
    let pll_on_bit: u32 = 1 << 24;
    let pll_ready_bit: u32 = 1 << 25;
    reg::RCC.set_CR(val | pll_on_bit);

    wait_for!(reg::RCC.CR() & pll_ready_bit == pll_ready_bit);
  }
}

// TODO(farcaller): this mod is pub as it's being used in peripheral_clock.rs.
//                  This is not the best design solution and a good reason to
//                  split RCC into distinct registers.
#[allow(missing_doc)]
pub mod reg {
  use util::volatile_cell::VolatileCell;

  pub enum SystemClockSwitch {
    SystemClockHSI = 0,
    SystemClockHSE = 1,
    SystemClockPLL = 2,
  }

  ioreg_old!(RCCReg: u32, CR, ICSCR, CFGR, CIR, AHBRSTR, APB2RSTR, APB1RSTR,
                          AHBENR, APB2ENR, APB1ENR, AHBLPENR, APB2LPENR,
                          APB1LPENR, CSR)
  reg_rw!(RCCReg, u32, CR,         set_CR,         CR)          // clock control
  reg_rw!(RCCReg, u32, ICSCR,      set_ICSCR,      ICSCR)       // internal clock sources calibration
  reg_rw!(RCCReg, u32, CFGR,       set_CFGR,       CFGR)        // clock configuration
  reg_rw!(RCCReg, u32, CIR,        set_CIR,        CIR)         // clock interrupt
  reg_rw!(RCCReg, u32, AHBRSTR,    set_AHBRSTR,    AHBRSTR)     // AHB peripheral reset
  reg_rw!(RCCReg, u32, APB2RSTR,   set_APB2RSTR,   APB2RSTR)    // APB2 peripheral reset
  reg_rw!(RCCReg, u32, APB1RSTR,   set_APB1RSTR,   APB1RSTR)    // APB1 peripheral reset
  reg_rw!(RCCReg, u32, AHBENR,     set_AHBENR,     AHBENR)      // AHB peripheral clock enable
  reg_rw!(RCCReg, u32, APB2ENR,    set_APB2ENR,    APB2ENR)     // APB2 peripheral clock enable
  reg_rw!(RCCReg, u32, APB1ENR,    set_APB1ENR,    APB1ENR)     // APB1 peripheral clock enable
  reg_rw!(RCCReg, u32, AHBLPENR,   set_AHBLPENR,   AHBLPENR)    // AHB peripheral clock enable in low power mode
  reg_rw!(RCCReg, u32, APB2LPENR,  set_APB2LPENR,  APB2LPENR)   // APB2 peripheral clock enable in low power mode
  reg_rw!(RCCReg, u32, APB1LPENR,  set_APB1LPENR,  APB1LPENR)   // APB1 peripheral clock enable in low power mode
  reg_rw!(RCCReg, u32, CSR,        set_CSR,        CSR)         // control/status

  ioreg_old!(FLASHReg: u32, ACR, PECR, PDKEYR, PEKEYR, PRGKEYR, OPTKEYR, SR,
                            OBR, WRPR)
  reg_rw!(FLASHReg, u32, ACR,     set_ACR,     ACR)     // access control
  reg_rw!(FLASHReg, u32, PECR,    set_PECR,    PECR)    // program/erase control
  reg_rw!(FLASHReg, u32, PDKEYR,  set_PDKEYR,  PDKEYR)  // power down key
  reg_rw!(FLASHReg, u32, PEKEYR,  set_PEKEYR,  PEKEYR)  // program/erase key
  reg_rw!(FLASHReg, u32, PRGKEYR, set_PRGKEYR, PRGKEYR) // program memory key
  reg_rw!(FLASHReg, u32, OPTKEYR, set_OPTKEYR, OPTKEYR) // option byte key
  reg_rw!(FLASHReg, u32, SR,      set_SR,      SR)      // status register
  reg_rw!(FLASHReg, u32, OBR,     set_OBR,     OBR)     // option byte
  reg_rw!(FLASHReg, u32, WRPR,    set_WRPR,    WRPR)    // write protection

  ioreg_old!(PWRReg: u32, CR, CSR)
  reg_rw!(PWRReg, u32, CR,  set_CR,  CR)    // power control
  reg_rw!(PWRReg, u32, CSR, set_CSR, CSR)   // power control/status

  extern {
    #[link_name="stm32l1_iomem_RCC"] pub static RCC: RCCReg;
    #[link_name="stm32l1_iomem_FLASH"] pub static FLASH: FLASHReg;
    #[link_name="stm32l1_iomem_PWR"] pub static PWR: PWRReg;
  }
}
