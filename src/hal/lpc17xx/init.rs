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

use hal::mem_init::init_data;
use hal::stack;

#[path="../../lib/ioreg.rs"] mod ioreg;
#[path="../../lib/wait_for.rs"] mod wait_for;

extern {
  static _eglobals: u32;
}

/// PLL clock source.
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
pub struct PLL0 {
  /// Specifies if PLL0 should be connected on boot.
  pub enabled: bool,
  /// PLL multiplier.
  pub m: u8,
  /// PLL divisor.
  pub n: u8,
  /// PLL output divisor.
  pub divisor: u8,
}

/// MCU clock configuration.
pub struct Clock {
  /// Clocking source.
  pub source: ClockSource,
  /// PLL configuration.
  pub pll: PLL0,
}

/// MCU configuration.
pub struct SysConf {
  /// Clock configuration.
  pub clock: Clock,
}

// TODO(farcaller): move to peripheral_clock?
static mut SystemClock: u32 = 0;

/// Returns system clock frequency according to configuration.
#[inline(always)]
pub fn system_clock() -> u32 {
  unsafe { SystemClock }
}

/// Performs the MCU initialization.
impl SysConf {
  pub fn setup(&self) {
    init_stack();
    init_data();
    init_clock(&self.clock);
  }
}

#[inline(always)]
fn init_stack() {
  stack::set_stack_limit((&_eglobals as *u32) as u32);
}

#[inline(always)]
fn init_clock(clock: &Clock) {
  let src_clock: u32 = match clock.source {
    Internal =>   4_000_000,
    Main(freq) => freq,
    RTC =>        32_000,
  };
  let dst_clock: u32;

  if clock.pll.enabled {
    match clock.source {
      Main(freq) => init_main_oscillator(freq),
      _ => (),
    }
    dst_clock = (src_clock * clock.pll.m as u32 * 2) / clock.pll.n as u32 / clock.pll.divisor as u32;
    init_flash_access(dst_clock);
    init_pll(&clock.pll, clock.source);
  } else {
    dst_clock = src_clock;
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
                        if freq > 40_000_000 { 2 } else
                        { 1 };
  let val = (num_clocks - 1) << 12;
  reg::FLASHCFG.set_value(val);
}

#[inline(always)]
fn wait_for_pll0stat_bit(bit: u32) {
  wait_for!(reg::PLL0STAT.value() & (1 << bit) == (1 << bit));
}

#[inline(always)]
fn write_pll0_changes() {
  reg::PLL0FEED.set_value(0xaa);
  reg::PLL0FEED.set_value(0x55);
}

#[inline(always)]
fn init_pll(pll: &PLL0, source: ClockSource) {
  match source {
    Internal => reg::CLKSRCSEL.set_value(0),
    Main(_)  => reg::CLKSRCSEL.set_value(1),
    RTC =>      reg::CLKSRCSEL.set_value(2),
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
  use lib::volatile_cell::VolatileCell;

  ioreg!(SCS: value)
  reg_rw!(SCS, value, set_value, value)
  ioreg!(FLASHCFG: value)
  reg_w!(FLASHCFG, set_value, value)
  ioreg!(PLL0CFG: value)
  reg_w!(PLL0CFG, set_value, value)
  ioreg!(PLL0CON: value)
  reg_w!(PLL0CON, set_value, value)
  ioreg!(PLL0FEED: value)
  reg_w!(PLL0FEED, set_value, value)
  ioreg!(PLL0STAT: value)
  reg_r!(PLL0STAT, value, value)
  ioreg!(CCLKCFG: value)
  reg_w!(CCLKCFG, set_value, value)
  ioreg!(CLKSRCSEL: value)
  reg_w!(CLKSRCSEL, set_value, value)

  extern {
    #[link_name="iomem_SCS"] pub static SCS: SCS;
    #[link_name="iomem_FLASHCFG"] pub static FLASHCFG: FLASHCFG;
    #[link_name="iomem_PLL0CFG"] pub static PLL0CFG: PLL0CFG;
    #[link_name="iomem_PLL0CON"] pub static PLL0CON: PLL0CON;
    #[link_name="iomem_CCLKCFG"] pub static CCLKCFG: CCLKCFG;
    #[link_name="iomem_PLL0FEED"] pub static PLL0FEED: PLL0FEED;
    #[link_name="iomem_CLKSRCSEL"] pub static CLKSRCSEL: CLKSRCSEL;
    #[link_name="iomem_PLL0STAT"] pub static PLL0STAT: PLL0STAT;
  }
}
