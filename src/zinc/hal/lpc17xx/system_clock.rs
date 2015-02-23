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
MCU initialication and clock configuration.

This module includes code for setting up the clock, flash, access time and
performing initial peripheral configuration.
*/

use core::option::Option::{self, Some, None};

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// PLL clock source.
#[derive(Copy)]
pub enum ClockSource {
  /// Internal resonator, 4MHz.
  Internal,
  /// External crystal with configurable frequency.
  Main(u32),
  /// Internal RTC resonator, 32KHz.
  RTC,
}

/// PLL0 configuration options.
///
/// Frequency is calculated as
///
/// ```
/// Fcco = (2 * m * Fin) / n
/// Fcpu = Fcco / divisor
/// ```
#[derive(Copy)]
pub struct PLL0 {
  /// PLL multiplier.
  pub m: u8,
  /// PLL divisor.
  pub n: u8,
  /// PLL output divisor.
  pub divisor: u8,
}

/// MCU clock configuration.
#[derive(Copy)]
pub struct Clock {
  /// Clocking source.
  pub source: ClockSource,
  /// PLL configuration.
  pub pll: Option<PLL0>,
}

// TODO(farcaller): move to peripheral_clock?
static mut SystemClock: u32 = 0;

/// Returns system clock frequency according to configuration.
#[inline(always)]
pub fn system_clock() -> u32 {
  unsafe { SystemClock }
}

/// Initialise the system clock.
#[inline(always)]
pub fn init_clock(clock: &Clock) {
  use self::ClockSource::*;
  let src_clock: u32 = match clock.source {
    Internal =>   4_000_000,
    Main(freq) => freq,
    RTC =>        32_000,
  };
  let dst_clock: u32;

  match clock.pll {
    Some(ref pll) => {
      match clock.source {
        Main(freq) => init_main_oscillator(freq),
        _ => (),
      }
      dst_clock = (src_clock * pll.m as u32 * 2) / pll.n as u32 / pll.divisor as u32;
      init_flash_access(dst_clock);
      init_pll(pll, &clock.source);
    },
    None => { dst_clock = src_clock; },
  }

  unsafe { SystemClock = dst_clock };
}

#[inline(always)]
fn init_main_oscillator(freq: u32) {
  let val: u32 = if freq > 15_000_000 { 1 << 4 } else { 0 } |
                 (1 << 5);

  reg::SCS.set_value(val);

  wait_for!(reg::SCS.value() & (1 << 6) == (1 << 6));
}

#[inline(always)]
fn init_flash_access(freq: u32) {
  let num_clocks: u32 = if freq > 100_000_000 { 6 } else
                        if freq > 80_000_000 { 5 } else
                        if freq > 60_000_000 { 4 } else
                        if freq > 40_000_000 { 3 } else
                        if freq > 20_000_000 { 2 } else
                        { 1 };
  let val = (num_clocks - 1) << 12;
  reg::FLASHCFG.set_value(val);
}

#[inline(always)]
fn wait_for_pll0stat_bit(bit: usize) {
  wait_for!(reg::PLL0STAT.value() & (1 << bit) == (1 << bit));
}

#[inline(always)]
fn write_pll0_changes() {
  reg::PLL0FEED.set_value(0xaa);
  reg::PLL0FEED.set_value(0x55);
}

#[inline(always)]
fn init_pll(pll: &PLL0, source: &ClockSource) {
  use self::ClockSource::*;
  match source {
    &Internal => reg::CLKSRCSEL.set_value(0),
    &Main(_)  => reg::CLKSRCSEL.set_value(1),
    &RTC =>      reg::CLKSRCSEL.set_value(2),
  }

  let val: u32 = ((pll.n as u32 - 1) << 16) | ((pll.m as u32 - 1) << 0);
  reg::PLL0CFG.set_value(val);
  write_pll0_changes();
  reg::PLL0CON.set_value(1);
  write_pll0_changes();
  wait_for_pll0stat_bit(24);
  reg::CCLKCFG.set_value((pll.divisor - 1) as u32);
  wait_for_pll0stat_bit(26);
  reg::PLL0CON.set_value(3);
  write_pll0_changes();
  wait_for_pll0stat_bit(25);
}

mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(SCS: u32, value);
  reg_rw!(SCS, u32, value, set_value, value);
  ioreg_old!(FLASHCFG: u32, value);
  reg_w!(FLASHCFG, u32, set_value, value);
  ioreg_old!(PLL0CFG: u32, value);
  reg_w!(PLL0CFG, u32, set_value, value);
  ioreg_old!(PLL0CON: u32, value);
  reg_w!(PLL0CON, u32, set_value, value);
  ioreg_old!(PLL0FEED: u32, value);
  reg_w!(PLL0FEED, u32, set_value, value);
  ioreg_old!(PLL0STAT: u32, value);
  reg_r!(PLL0STAT, u32, value, value);
  ioreg_old!(CCLKCFG: u32, value);
  reg_w!(CCLKCFG, u32, set_value, value);
  ioreg_old!(CLKSRCSEL: u32, value);
  reg_w!(CLKSRCSEL, u32, set_value, value);

  extern {
    #[link_name="lpc17xx_iomem_SCS"] pub static SCS: SCS;
    #[link_name="lpc17xx_iomem_FLASHCFG"] pub static FLASHCFG: FLASHCFG;
    #[link_name="lpc17xx_iomem_PLL0CFG"] pub static PLL0CFG: PLL0CFG;
    #[link_name="lpc17xx_iomem_PLL0CON"] pub static PLL0CON: PLL0CON;
    #[link_name="lpc17xx_iomem_CCLKCFG"] pub static CCLKCFG: CCLKCFG;
    #[link_name="lpc17xx_iomem_PLL0FEED"] pub static PLL0FEED: PLL0FEED;
    #[link_name="lpc17xx_iomem_CLKSRCSEL"] pub static CLKSRCSEL: CLKSRCSEL;
    #[link_name="lpc17xx_iomem_PLL0STAT"] pub static PLL0STAT: PLL0STAT;
  }
}
