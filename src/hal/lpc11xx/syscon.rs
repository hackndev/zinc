// Zinc, the bare metal stack for rust.
// Copyright 2015 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
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

//! Routines for initialization and system configuration of NXP LPC11xx.
//!
//! This module includes code for setting up the clock, flash, access time and
//! performing initial peripheral configuration.

use super::regs;

/// Interrupt vectors source.
#[derive(PartialEq, Debug)]
pub enum ISRLocation {
    Bootloader = 0,
    RAM = 1,
    Flash = 2
}

/// Returns the current source of interrupt vectors.
pub fn get_isr_location() -> ISRLocation {
  match regs::SYSCON().sysmemremap.map() {
    regs::SYSCON_sysmemremap_map::BOOT_LOADER_MODE_IN => ISRLocation::Bootloader,
    regs::SYSCON_sysmemremap_map::USER_RAM_MODE_INTER => ISRLocation::RAM,
    regs::SYSCON_sysmemremap_map::USER_FLASH_MODE_INT => ISRLocation::Flash,
  }
}

/// Re-maps interrupt vectors to either RAM or Flash.
pub fn set_isr_location(loc: ISRLocation) {
  regs::SYSCON().sysmemremap.ignoring_state().set_map(match loc {
    ISRLocation::Bootloader => regs::SYSCON_sysmemremap_map::BOOT_LOADER_MODE_IN,
    ISRLocation::RAM        => regs::SYSCON_sysmemremap_map::USER_RAM_MODE_INTER,
    _ => panic!(),
  });
}

pub fn init_system_clock() {
  regs::SYSCON().pdruncfg
      .set_sysosc_pd(regs::SYSCON_pdruncfg_sysosc_pd::POWERED);
  regs::SYSCON().sysoscctrl.ignoring_state()
      .set_bypass(regs::SYSCON_sysoscctrl_bypass::NOBYPASS)
      .set_freqrange(regs::SYSCON_sysoscctrl_freqrange::LOW);
  regs::SYSCON().syspllclksel.ignoring_state()
      .set_sel(regs::SYSCON_syspllclksel_sel::SYSTEM_OSCILLATOR);

  regs::SYSCON().syspllclkuen.ignoring_state()
      .set_ena(regs::SYSCON_syspllclkuen_ena::UPDATE_CLOCK_SOURCE);
  regs::SYSCON().syspllclkuen.ignoring_state()
      .set_ena(regs::SYSCON_syspllclkuen_ena::NO_CHANGE);
  regs::SYSCON().syspllclkuen.ignoring_state()
      .set_ena(regs::SYSCON_syspllclkuen_ena::UPDATE_CLOCK_SOURCE);

  loop {
    if regs::SYSCON().syspllclkuen.ena() == regs::SYSCON_syspllclkuen_ena::UPDATE_CLOCK_SOURCE {
      break
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use volatile_cell::{VolatileCellReplayer, set_replayer};
  use expectest::prelude::*;
  use expectest;

  #[test]
  fn returns_isr_location() {
    let mut replayer = VolatileCellReplayer::new();
    set_replayer(&mut replayer);

    expect_volatile_read!(replayer, 0x4004_8000, 0b10);

    expect!(get_isr_location()).to(be_equal_to(ISRLocation::Flash));

    expect_replayer_valid!(replayer);
  }

  #[test]
  fn sets_isr_location() {
    let mut replayer = VolatileCellReplayer::new();
    set_replayer(&mut replayer);

    expect_volatile_write!(replayer, 0x4004_8000, 0b00);

    set_isr_location(ISRLocation::Bootloader);

    expect_replayer_valid!(replayer);
  }

  #[test]
  fn initialize_system_clock() {
    let mut replayer = VolatileCellReplayer::new();
    set_replayer(&mut replayer);

    // read PDRUNCFG, returns reset value
    expect_volatile_read!(replayer,  0x4004_8238, 0x0000_EDF0);
    // write PDRUNCFG, set SYSOSC_PD to POWERED
    expect_volatile_write!(replayer, 0x4004_8238, 0x0000_EDD0);

    // write SYSOSCCTRL, set BYPASS to off, FREQRANGE 1-20MHz
    expect_volatile_write!(replayer, 0x4004_8020, 0x0000_0000);

    // write SYSPLLCLKSEL, set SEL to system oscillator
    expect_volatile_write!(replayer, 0x4004_8040, 0x0000_0001);

    // write SYSPLLCLKUEN, set update/no change/update
    expect_volatile_write!(replayer, 0x4004_8044, 0x0000_0001);
    expect_volatile_write!(replayer, 0x4004_8044, 0x0000_0000);
    expect_volatile_write!(replayer, 0x4004_8044, 0x0000_0001);

    // poll-read SYSPLLCLKUEN until returns update
    expect_volatile_read!(replayer,  0x4004_8044, 0x0000_0000);
    expect_volatile_read!(replayer,  0x4004_8044, 0x0000_0000);
    expect_volatile_read!(replayer,  0x4004_8044, 0x0000_0000);
    expect_volatile_read!(replayer,  0x4004_8044, 0x0000_0001);

    init_system_clock();

    expect_replayer_valid!(replayer);
  }
}
