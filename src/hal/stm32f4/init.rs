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

//! Routines for initialization of STM32F4.
//!
//! This module includes code for setting up the clock, flash, access time and
//! performing initial peripheral configuration.

use hal::mem_init::init_data;
use core::intrinsics::abort;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// System clock source.
#[derive(Clone, Copy)]
pub enum SystemClockSource {
  /// High-speed internal oscillator, 16MHz.
  SystemClockHSI,
  /// High-speed external oscillator with configurable frequency.
  SystemClockHSE(u32),
  /// PLL.
  SystemClockPLL(PLLConf),
}

/// PLL clock source. Applies to both PLL and PLLI2S.
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct ClockConf {
  /// Clocking source.
  pub source: SystemClockSource,
}

/// MCU configuration.
#[derive(Clone, Copy)]
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
    use self::SystemClockSource::*;
    use self::PLLClockSource::*;

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
          self.set_system_clock(reg::SystemClockSwitch::SystemClockHSE);
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
        self.set_system_clock(reg::SystemClockSwitch::SystemClockPLL);
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
    use self::PLLClockSource::*;

    // TODO(farcaller): cmsis code overrides reserved bits in here, is that ok?
    let val = reg::RCC.PLLCFGR();
    let mask: u32 = !0b0000_1111_0_1_0000_11_0_111111111_111111;
    let bits: u32 =
      self.m as u32 |
      ((self.n as u32) << 6) |
      (match self.p {
        2 => 0b00u32,
        4 => 0b01u32,
        6 => 0b10u32,
        8 => 0b11u32,
        _ => unsafe { abort() },
      } << 16) |
      (match self.source {
        PLLClockHSI    => 0u32,
        PLLClockHSE(_) => 1u32,
      } << 22) |
      ((self.q as u32) << 24);

    reg::RCC.set_PLLCFGR(val & mask | bits);

    self.enable_pll();
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
#[allow(missing_docs)]
pub mod reg {
  use util::volatile_cell::VolatileCell;

  #[derive(Clone, Copy)]
  pub enum SystemClockSwitch {
    SystemClockHSI = 0,
    SystemClockHSE = 1,
    SystemClockPLL = 2,
  }

  ioreg_old!(RCCReg: u32, CR, PLLCFGR, CFGR, CIR, AHB1RSTR, AHB2RSTR, AHB3RSTR,
                      _pad_0, APB1RSTR, APB2RSTR, _pad_1, _pad_2, AHB1ENR,
                      AHB2ENR, AHB3ENR, _pad_3, APB1ENR, APB2ENR, _pad_4,
                      _pad_5, AHB1LPENR, AHB2LPENR, AHB3LPENR, _pad_6,
                      APB1LPENR, APB2LPENR, _pad_7, _pad_8, BDCR, CSR, _pad_9,
                      _pad_10, SSCGR, PLLI2SCFGR);
  reg_rw!(RCCReg, u32, CR,         set_CR,         CR);
  reg_rw!(RCCReg, u32, PLLCFGR,    set_PLLCFGR,    PLLCFGR);
  reg_rw!(RCCReg, u32, CFGR,       set_CFGR,       CFGR);
  reg_rw!(RCCReg, u32, CIR,        set_CIR,        CIR);
  reg_rw!(RCCReg, u32, AHB1RSTR,   set_AHB1RSTR,   AHB1RSTR);
  reg_rw!(RCCReg, u32, AHB2RSTR,   set_AHB2RSTR,   AHB2RSTR);
  reg_rw!(RCCReg, u32, AHB3RSTR,   set_AHB3RSTR,   AHB3RSTR);
  reg_rw!(RCCReg, u32, APB1RSTR,   set_APB1RSTR,   APB1RSTR);
  reg_rw!(RCCReg, u32, APB2RSTR,   set_APB2RSTR,   APB2RSTR);
  reg_rw!(RCCReg, u32, AHB1ENR,    set_AHB1ENR,    AHB1ENR);
  reg_rw!(RCCReg, u32, AHB2ENR,    set_AHB2ENR,    AHB2ENR);
  reg_rw!(RCCReg, u32, AHB3ENR,    set_AHB3ENR,    AHB3ENR);
  reg_rw!(RCCReg, u32, APB1ENR,    set_APB1ENR,    APB1ENR);
  reg_rw!(RCCReg, u32, APB2ENR,    set_APB2ENR,    APB2ENR);
  reg_rw!(RCCReg, u32, AHB1LPENR,  set_AHB1LPENR,  AHB1LPENR);
  reg_rw!(RCCReg, u32, AHB2LPENR,  set_AHB2LPENR,  AHB2LPENR);
  reg_rw!(RCCReg, u32, AHB3LPENR,  set_AHB3LPENR,  AHB3LPENR);
  reg_rw!(RCCReg, u32, APB1LPENR,  set_APB1LPENR,  APB1LPENR);
  reg_rw!(RCCReg, u32, APB2LPENR,  set_APB2LPENR,  APB2LPENR);
  reg_rw!(RCCReg, u32, BDCR,       set_BDCR,       BDCR);
  reg_rw!(RCCReg, u32, CSR,        set_CSR,        CSR);
  reg_rw!(RCCReg, u32, SSCGR,      set_SSCGR,      SSCGR);
  reg_rw!(RCCReg, u32, PLLI2SCFGR, set_PLLI2SCFGR, PLLI2SCFGR);

  ioreg_old!(FLASHReg: u32, ACR, KEYR, OPTKEYR, SR, CR, OPTCR);
  reg_rw!(FLASHReg, u32, ACR,   set_ACR,     ACR);
  reg_w!(FLASHReg,  u32,        set_KEYR,    KEYR);
  reg_w!(FLASHReg,  u32,        set_OPTKEYR, OPTKEYR);
  reg_rw!(FLASHReg, u32, SR,    set_SR,      SR);
  reg_rw!(FLASHReg, u32, CR,    set_CR,      CR);
  reg_rw!(FLASHReg, u32, OPTCR, set_OPTCR,   OPTCR);

  ioreg_old!(PWRReg: u32, CR, CSR);
  reg_rw!(PWRReg, u32, CR,  set_CR,  CR);
  reg_rw!(PWRReg, u32, CSR, set_CSR, CSR);

  extern {
    #[link_name="stm32f4_iomem_RCC"] pub static RCC: RCCReg;
    #[link_name="stm32f4_iomem_FLASH"] pub static FLASH: FLASHReg;
    #[link_name="stm32f4_iomem_PWR"] pub static PWR: PWRReg;
  }
}
