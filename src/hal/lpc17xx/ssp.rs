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
SSP configuration.

Currently supports only SPI mode. Note that `SPI` is not the same peripheral and
it's currently not supported at all.
*/

use core::intrinsics::abort;

use hal::lpc17xx::peripheral_clock::{PeripheralClock, SSP0Clock, SSP1Clock};
use hal::lpc17xx::system_clock::system_clock;
use hal::pin::PinConf_;
use hal::spi;

#[path="../../util/ioreg.rs"] mod ioreg;

/// SPI configuration.
///
/// This configuration doesn't manage the chip-select pin, it must be configured
/// and used externally via GPIOConf.
pub struct SPIConf {
  /// Peripheral to use, mcu-specific.
  pub peripheral: SSPPeripheral,
  /// Number of bits per transfer, commonly 8.
  pub bits: u8,
  /// SPI mode, see http://en.wikipedia.org/wiki/Serial_Peripheral_Interface_Bus#Mode_numbers for explanation.
  pub mode: u8,
  /// SPI bus frequency, obiviously must be lover than core clock.
  ///
  /// The divisor is currently hardcoded and is equal to 1.
  pub frequency: u32,

  /// MOSI pin to use, supports NotConnected pin.
  pub mosi: PinConf_,
  /// MISO pin to use, supports NotConnected pin.
  pub miso: PinConf_,
  /// SCLK pin to use, supports NotConnected pin.
  pub sclk: PinConf_,
}

impl SPIConf {
  /// Returns a platform-specific object, that implements SPI trait.
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

/// Opaque object that manages the configured peripheral.
#[allow(dead_code)]
pub struct SSP {
  peripheral: SSPPeripheral, // TODO(farcaller): clean up the warning
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
  #[allow(non_snake_case)]
  fn set_format(&self, bits: u8, mode: u8) {
    let slave = false;

    self.disable();
    if !(bits >= 4 && bits <= 16) || mode > 3 {
      unsafe { abort() };
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

  fn set_frequency(&self, freq: u32) {
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
    unsafe { abort() };
  }

  fn disable(&self) {
    let old_reg: u32 = self.reg.CR1();
    let new_reg: u32 = old_reg & 0b1101;
    self.reg.set_CR1(new_reg);
  }

  fn enable(&self) {
    let old_reg: u32 = self.reg.CR1();
    let new_reg: u32 = old_reg | 0b0010;
    self.reg.set_CR1(new_reg);
  }

  fn readable(&self) -> bool {
    let val: u32 = self.reg.SR();

    (val & 0b00100) == 0b00100
  }

  fn writeable(&self) -> bool {
    let val: u32 = self.reg.SR();

    (val & 0b00010) == 0b00010
  }

  fn written(&self) -> bool {
    let val: u32 = self.reg.SR();

    (val & 0b10000) == 0
  }
}

impl spi::Spi for SSP {
  fn write(&self, value: u8) {
    loop { if self.writeable() {
      break;
    } }
    self.reg.set_DR(value as u32);
    loop { if self.written() {
        break;
    } }
  }

  fn read(&self) -> u8 {
    loop {
      if self.readable() {
        break;
      }
    }
    (self.reg.DR() & 0xff) as u8
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(SSP: u32, CR0, CR1, DR, SR, CPSR, IMSC, RIS, MIS, ICR, DMACR)
  reg_rw!(SSP, u32, CR0,   set_CR0,   CR0)
  reg_rw!(SSP, u32, CR1,   set_CR1,   CR1)
  reg_rw!(SSP, u32, DR,    set_DR,    DR)
  reg_r!( SSP, u32, SR,               SR)
  reg_rw!(SSP, u32, CPSR,  set_CPSR,  CPSR)
  reg_rw!(SSP, u32, IMSC,  set_IMSC,  IMSC)
  reg_rw!(SSP, u32, RIS,   set_RIS,   RIS)
  reg_rw!(SSP, u32, MIS,   set_MIS,   MIS)
  reg_rw!(SSP, u32, ICR,   set_ICR,   ICR)
  reg_rw!(SSP, u32, DMACR, set_DMACR, DMACR)

  extern {
    #[link_name="lpc17xx_iomem_SSP0"] pub static SSP0: SSP;
    #[link_name="lpc17xx_iomem_SSP1"] pub static SSP1: SSP;
  }
}
