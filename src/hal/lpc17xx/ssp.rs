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

use core::*;
use hal::lpc17xx::peripheral_clock::{PeripheralClock, SSP0Clock, SSP1Clock};
use hal::lpc17xx::init::system_clock;
use hal::pin::PinConf_;
use hal::spi;
use core::fail::abort;

#[path="../../lib/ioreg.rs"] mod ioreg;

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg_cell!(SSP: CR0, CR1, DR, SR, CPSR, IMSC, RIS, MIS, ICR, DMACR)
  reg_cell_rw!(SSP, CR0,   set_CR0,   CR0)
  reg_cell_rw!(SSP, CR1,   set_CR1,   CR1)
  reg_cell_rw!(SSP, DR,    set_DR,    DR)
  reg_cell_r!( SSP, SR,               SR)
  reg_cell_rw!(SSP, CPSR,  set_CPSR,  CPSR)
  reg_cell_rw!(SSP, IMSC,  set_IMSC,  IMSC)
  reg_cell_rw!(SSP, RIS,   set_RIS,   RIS)
  reg_cell_rw!(SSP, MIS,   set_MIS,   MIS)
  reg_cell_rw!(SSP, ICR,   set_ICR,   ICR)
  reg_cell_rw!(SSP, DMACR, set_DMACR, DMACR)

  extern {
    #[link_name="iomem_SSP0"] pub static SSP0: SSP;
    #[link_name="iomem_SSP1"] pub static SSP1: SSP;
  }
}

pub struct SPIConf {
  pub peripheral: SSPPeripheral,
  pub bits: u8,
  pub mode: u8,
  pub frequency: u32,

  pub mosi: PinConf_,
  pub miso: PinConf_,
  pub sclk: PinConf_,
}

impl SPIConf {
  pub fn setup(&self) -> SSP {
    let ssp = SSP {
      peripheral: self.peripheral,
      reg: self.peripheral.reg(),
    };

    let clock = self.peripheral.peripheral_clock();
    clock.enable();
    clock.set_divisor(1);
    ssp.set_format(self.bits, self.mode);
    ssp.set_frequency(self.frequency);

    self.mosi.setup();
    self.miso.setup();
    self.sclk.setup();

    ssp
  }
}

pub struct SSP {
  peripheral: SSPPeripheral,
  reg: &'static reg::SSP,
}

pub enum SSPPeripheral {SSP0, SSP1}

impl SSPPeripheral {
  fn reg(self) -> &reg::SSP {
    match self {
      SSP0 => &reg::SSP0,
      SSP1 => &reg::SSP1,
    }
  }

  fn peripheral_clock(self) -> PeripheralClock {
    match self {
      SSP0 => SSP0Clock,
      SSP1 => SSP1Clock,
    }
  }
}

impl SSP {
  #[allow(uppercase_variables)]
  pub fn set_format(&self, bits: u8, mode: u8) {
    let slave = false;

    self.disable();
    if !(bits >= 4 && bits <= 16) || mode > 3 {
      fail::abort();
    }

    let polarity = mode & 0x2 != 0;
    let phase = mode & 0x1 != 0;

    let DSS: u32 = bits as u32 - 1;            // DSS (data select size)
    let SPO: u32 = if polarity { 1 } else { 0 };  // SPO - clock out polarity
    let SPH: u32 = if phase { 1 } else { 0 };     // SPH - clock out phase

    let FRF: u32 = 0;                   // FRF (frame format) = SPI
    let old_reg0: u32 = self.reg.CR0();
    let new_reg0: u32 = old_reg0 & 0xffffff00 |
      (DSS << 0) |
      (FRF << 4) |
      (SPO << 6) |
      (SPH << 7) ;
    self.reg.set_CR0(new_reg0);

    let LBM: u32 = 0;
    let SSE: u32 = 0;
    let MS:  u32 = if slave { 1 } else { 0 };
    let SOD: u32 = 0;
    let new_reg1: u32 =
      (LBM << 0) |
      (SSE << 1) |
      (MS  << 2) |
      (SOD << 0);
    self.reg.set_CR1(new_reg1);

    self.enable();
  }

  pub fn set_frequency(&self, freq: u32) {
    self.disable();

    let mut prescaler: u32 = 2;

    while prescaler <= 254 {
      let prescale_hz: u32 = system_clock() / prescaler;

      // calculate the divider
      let divider: u32 = ((prescale_hz as f32 / freq as f32) + 0.5f32) as u32;

      // check we can support the divider
      if divider < 256 {
          // prescaler
          self.reg.set_CPSR(prescaler);

          // divider
          let old_reg: u32 = self.reg.CR0();
          let new_reg: u32 = old_reg & 0xff |
            ((divider-1) << 8);
          self.reg.set_CR0(new_reg);
          self.enable();
          return
      }
      prescaler += 2;
    }
    abort();
  }

  pub fn disable(&self) {
    let old_reg: u32 = self.reg.CR1();
    let new_reg: u32 = old_reg & 0b1101;
    self.reg.set_CR1(new_reg);
  }

  pub fn enable(&self) {
    let old_reg: u32 = self.reg.CR1();
    let new_reg: u32 = old_reg | 0b0010;
    self.reg.set_CR1(new_reg);
  }

  pub fn readable(&self) -> bool {
    let val: u32 = self.reg.SR();

    (val & 0b00100) == 0b00100
  }

  pub fn writeable(&self) -> bool {
    let val: u32 = self.reg.SR();

    (val & 0b00010) == 0b00010
  }

  pub fn written(&self) -> bool {
    let val: u32 = self.reg.SR();

    (val & 0b10000) == 0
  }
}

impl spi::SPI for SSP {
  #[no_split_stack]
  fn write(&self, value: u8) {
    loop { if self.writeable() {
      break;
    } }
    self.reg.set_DR(value as u32);
    loop { if self.written() {
        break;
    } }
  }

  #[no_split_stack]
  fn read(&self) -> u8 {
    loop {
      if self.readable() {
        break;
      }
    }
    (self.reg.DR() & 0xff) as u8
  }
}
