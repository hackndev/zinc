// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
// Adapted from stm32f4/init.rs for stm32f7 by Dave Hylands <dhylands@gmail.com>
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

//! Routines for initialization of STM32F7.
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

        if sysfreq > 180_000_000 {
          // frequencies above 180 MHz require over-drive mode. So when that's
          // implemented then we can drop this.
          unsafe { abort() };
        }

        // TODO(farcaller): this doesn't really belong here.
        self.setup_flash(sysfreq);
        self.set_system_clock(reg::SystemClockSwitch::SystemClockPLL);
        unsafe {
          SystemClock = sysfreq;
          APBLowClock = sysfreq / apb_low_divisor as u32;
        };
      },
    };

    // The DFU bootloader changes the clocksource register from its default power
    // on reset value, so we set it back here, so the clocksources are the same
    // whether we were started from DFU or from a power on reset.

    reg::RCC.dckcfgr2.set_val(0);
  }

  fn setup_flash(&self, freq: u32) {
    reg::FLASH.acr
      .set_art_on(true)
      .set_prefetch_on(true)
      .set_latency(match freq/1_000_000 {  // wait states are calculated for 2.7-3.6V range
        0...30    => 0,
        31...60   => 1,
        61...90   => 2,
        90...120  => 3,
        121...150 => 4,
        151...180 => 5,
        181...210 => 6,
        211...216 => 7,
        _        => unsafe { abort() },
      });
  }

  fn enable_hse(&self) {
    reg::RCC.cr.set_hse_on(true);
    wait_for!(reg::RCC.cr.hse_ready());
  }

  fn set_system_clock(&self, clock: reg::SystemClockSwitch) {
    reg::RCC.cfgr.set_system_clock(clock as u32);
  }

  fn set_clock_divisors(&self, ahb: u16, apb_low_speed: u8, apb_hi_speed: u8) {
    reg::RCC.cfgr
      .set_ahb_prescaler(match ahb {
        1   => 0b0000,
        2   => 0b1000,
        4   => 0b1001,
        8   => 0b1010,
        16  => 0b1011,
        64  => 0b1100,
        128 => 0b1101,
        256 => 0b1110,
        512 => 0b1111,
        _   => unsafe { abort() }})
      .set_apb1_prescaler(match apb_low_speed {
        1   => 0b000,
        2   => 0b100,
        4   => 0b101,
        8   => 0b110,
        16  => 0b111,
        _   => unsafe { abort() }})
      .set_apb2_prescaler(match apb_hi_speed {
        1   => 0b000,
        2   => 0b100,
        4   => 0b101,
        8   => 0b110,
        16  => 0b111,
        _   => unsafe { abort() }});
  }
}

impl PLLConf {
  fn setup(&self) {
    use self::PLLClockSource::*;
    reg::RCC.pllcfgr
      .set_pll_div_factor(self.m as u32)
      .set_pll_mul_factor(self.n as u32)
      .set_sysclk_prescaler(match self.p {
        2 => 0b00u32,
        4 => 0b01u32,
        6 => 0b10u32,
        8 => 0b11u32,
        _ => unsafe { abort() }
      })
      .set_pll_clock_source_hse(match self.source {
        PLLClockHSI    => false,
        PLLClockHSE(_) => true,
      })
      .set_usb_prescaler(self.q as u32);

    self.enable_pll();
  }

  fn enable_pll(&self) {
    reg::RCC.cr.set_pll_on(true);
    wait_for!(reg::RCC.cr.pll_ready());
  }
}

// TODO(farcaller): this mod is pub as it's being used in peripheral_clock.rs.
//                  This is not the best design solution and a good reason to
//                  split RCC into distinct registers.
#[allow(missing_docs)]
pub mod reg {
  use volatile_cell::VolatileCell;

  #[derive(Clone, Copy)]
  pub enum SystemClockSwitch {
    SystemClockHSI = 0,
    SystemClockHSE = 1,
    SystemClockPLL = 2,
  }

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
      26 => pll_i2s_on : rw,
      27 => pll_i2s_ready : ro,
      28 => pll_sai_on : rw,
      29 => pll_sai_ready : ro,
    },
    0x04 => reg32 pllcfgr {         // PLL configuration
      5..0   => pll_div_factor : rw,    // PLLM
      14..6  => pll_mul_factor : rw,    // PLLN
      17..16 => sysclk_prescaler : rw,  // PLLP
      22     => pll_clock_source_hse : rw,
      27..24 => usb_prescaler : rw,     // PLLQ
    },
    0x08 => reg32 cfgr {        // clock configuration
      1..0   => system_clock : rw,
      3..2   => system_clock_status: ro,
      7..4   => ahb_prescaler : rw,
      12..10 => apb1_prescaler : rw,
      15..13 => apb2_prescaler : rw,
      20..16 => rtc_prescaler : rw,
      22..21 => mco1 : rw,
      23     => i2s_clock_souce_external : rw,
      26..24 => mco1_prescaler : rw,
      29..27 => mco2_prescaler : rw,
      31..30 => mco2 : rw,
    },
    0x0C => reg32 cir {         // clock interrupt
      31..0 => clock_interrupt : rw,
    },
    0x10 => reg32 ahb1rstr {    // AHB1 peripheral reset
      31..0 => reset : rw,
    },
    0x14 => reg32 ahb2rstr {    // AHB2 peripheral reset
      31..0 => reset : rw,
    },
    0x18 => reg32 ahb3rstr {    // AHB3 peripheral reset
      31..0 => reset : rw,
    },
    0x20 => reg32 apb1rstr {    // APB1 peripheral reset
      31..0 => reset : rw,
    },
    0x24 => reg32 apb2rstr {    // APB2 peripheral reset
      31..0 => reset : rw,
    },
    0x30 => reg32 ahb1enr {     // AHB1 peripheral clock enable
      31..0 => enable : rw,
    },
    0x34 => reg32 ahb2enr {     // AHB2 peripheral clock enable
      31..0 => enable : rw,
    },
    0x38 => reg32 ahb3enr {     // AHB3 peripheral clock enable
      31..0 => enable : rw,
    },
    0x40 => reg32 apb1enr {     // APB1 peripheral clock enable
      31..0 => enable : rw,
    },
    0x44 => reg32 apb2enr {     // APB2 peripheral clock enable
      31..0 => enable : rw,
    },
    0x50 => reg32 ahb1lpenr {     // AHB1 low power peripheral clock enable
      31..0 => enable : rw,
    },
    0x54 => reg32 ahb2lpenr {     // AHB2 low powerperipheral clock enable
      31..0 => enable : rw,
    },
    0x58 => reg32 ahb3lpenr {     // AHB3 low powerperipheral clock enable
      31..0 => enable : rw,
    },
    0x60 => reg32 apb1lpenr {     // APB1 low powerperipheral clock enable
      31..0 => enable : rw,
    },
    0x64 => reg32 apb2lpenr {     // APB2 low powerperipheral clock enable
      31..0 => enable : rw,
    },
    0x70 => reg32 bdcr {          // Backup Domain Control Register
      0    => lse_on : rw,
      1    => lse_ready : ro,
      2    => lse_bypass : rw,
      4..3 => lse_drive : rw,
      9..8 => rtc_source : rw,
      15   => rtc_on : rw,
      16   => backup_reset : rw,
    },
    0x74 => reg32 csr {           // Clock Control & Status
      0 => lsi_on : rw,
      1 => lsi_ready : ro,
      24 => remove_reset : rw,
      25 => bor_reset : ro,
      26 => pin_reset : ro,
      27 => por_pdr_reset : ro,
      28 => software_reset : ro,
      29 => independent_watchdog_reset : ro,
      30 => window_watchdog_reset : ro,
      31 => low_power_reset : ro,
    },
    0x80 => reg32 sscgr {           // Spread Spectrum Clock Generation Register
      12..0  => modulation_period : rw,
      27..13 => increment_step : rw,
      30     => spread_select : rw,
      31     => ss_modulation_on : rw,
    },
    0x84 => reg32 plli2scfgr {      // PLL I2S Configuration Register
      14..6  => i2s_vco_mul_factor : rw,    // PLLI2SN
      17..16 => i2s_spdifrx_prescaler : rw, // PLLI2SP
      27..24 => i2s_sai_prescaler : rw,     // PLLI2SQ
      30..28 => i2s_prescaler : rw,         // PLLI2SR
    },
    0x88 => reg32 pllsaicfgr {      // PLL SAI Configuration Register
      14..6  => sai_vco_div_factor : rw,  // PLLSAIN
      17..16 => sai48_prescaler : rw,     // PLLSAIP
      27..24 => sai_prescaler : rw,       // PLLSAIQ
      30..28 => sai_lcd_prescaler : rw,   // PLLSAIR
    },
    0x8C => reg32 dckcfgr1 {        // Dedicated Clocks Configuration Register 1
      4..0   => i2s_sai1_prescaler : rw,
      12..8  => sai_sai1_prescaler : rw,
      17..16 => sai_lcd_prescaler : rw,
      21..20 => sai1_clock_source : rw,
      23..22 => sai2_clock_source : rw,
      24     => tim_prescaler : rw
    },

    // There doesn't seem to get a set/set of the entire register, so we just
    // describe it as 32-bits for now.
    0x90 => reg32 dckcfgr2 {
      31..0 => val : rw,
    },
//    0x90 => reg32 dckcfgr2 {        // Dedicated Clocks Configuration Register 2
//      1..0   => uart1_clock_source : rw,
//      3..2   => uart2_clock_source : rw,
//      5..4   => uart3_clock_source : rw,
//      7..6   => uart4_clock_source : rw,
//      9..8   => uart5_clock_source : rw,
//      11..10 => uart6_clock_source : rw,
//      13..12 => uart7_clock_source : rw,
//      15..14 => uart8_clock_source : rw,
//      17..16 => i2c1_clock_source : rw,
//      19..18 => i2c2_clock_source : rw,
//      21..20 => i2c3_clock_source : rw,
//      23..22 => i2c4_clock_source : rw,
//      25..24 => lptim1_clock_source : rw,
//      26     => cec_clock_source : rw,
//      27     => ck48_clock_source : rw,
//      28     => sdmmc_clock_source : rw,
//    },
  });

  ioregs!(FLASH = {
    0x00 => reg32 acr {     // access control
      3..0 => latency : rw,
      8    => prefetch_on : rw,
      9    => art_on : rw,
      11   => art_reset : rw,
    },
    0x04 => reg32 keyr {    // Key register
      31..0 => key : wo,
    },
    0x08 => reg32 optkeyr { // Option Key Register
      31..0 => key,
    },
    0x0c => reg32 sr {      // Status Register
      31..0 => status : rw,
    },
    0x10 => reg32 cr {      // Control Register
      31..0 => control : rw,
    },
    0x14 => reg32 optcr {   // Option Control Register
      31..0 => reg : rw,
    },
    0x18 => reg32 optcr1 {   // Option Control Register 1
      31..16 => boot_addr1 : rw,
      15..0  => boot_addr0 : rw,
    },
  });

  ioregs!(PWR = {
    0x0 => reg32 cr1 {        // Power Control Register 1
      31..0 => control : rw,
    },
    0x4 => reg32 csr1 {       // Power Control/Status Regiuster 1
      31..0 => status : rw,
    },
    0x8 => reg32 cr2 {        // Power Control Register 2
      31..0 => control : rw,
    },
    0xC => reg32 csr2 {       // Power Control/Status Regiuster 1
      31..0 => status : rw,
    },
  });

  extern {
    #[link_name="stm32f7_iomem_RCC"] pub static RCC: RCC;
    #[link_name="stm32f7_iomem_FLASH"] pub static FLASH: FLASH;
    #[link_name="stm32f7_iomem_PWR"] pub static PWR: PWR;
  }
}
