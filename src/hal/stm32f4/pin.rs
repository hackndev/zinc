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

//! Pin configuration for ST STM32F4.
//!
//! Some pins that could be configured here may be missing from actual MCU
//! depending on the package.

use super::peripheral_clock;
use core::intrinsics::abort;

#[path="../../lib/ioreg.rs"] mod ioreg;

/// Available port names.
pub enum Port {
  PortA,
  PortB,
  PortC,
  PortD,
  PortE,
  PortF,
  PortG,
  PortH,
  PortI,
}

/// Pin functions.
pub enum Function {
  GPIOIn      = 0,
  GPIOOut     = 1,
  AltFunction = 2,
  Analog      = 3,
}

/// Pin configuration.
///
/// This structure shouldn't be used directly, pinmap.rs, available via pin::map
/// has all possible pin configurations.
pub struct PinConf {
  /// Pin port, mcu-specific.
  pub port: Port,
  /// Pin number.
  pub pin: u8,
  /// Pin function, mcu-specific.
  pub function: Function,
}

impl Port {
  fn clock(self) -> peripheral_clock::PeripheralClock {
    match self {
      PortA => peripheral_clock::GPIOAClock,
      PortB => peripheral_clock::GPIOBClock,
      PortC => peripheral_clock::GPIOCClock,
      PortD => peripheral_clock::GPIODClock,
      PortE => peripheral_clock::GPIOEClock,
      PortF => peripheral_clock::GPIOFClock,
      PortG => peripheral_clock::GPIOGClock,
      PortH => peripheral_clock::GPIOHClock,
      PortI => peripheral_clock::GPIOIClock,
    }
  }
}

impl PinConf {
  #[no_split_stack]
  #[inline(always)]
  pub fn setup(&self) {
    self.port.clock().enable();  // TODO(farcaller): should be done once per port

    let offset: uint = self.pin as uint * 2;
    let gpreg = self.get_reg();

    let bits: u32 = match self.function {
      GPIOOut => 0b01 << offset,
      GPIOIn  => 0b00 << offset,
      _       => unsafe { abort() },  // FIXME(farcaller): not implemented
    };
    let mask: u32 = !(0b11 << offset);
    let val: u32 = gpreg.MODER();

    gpreg.set_MODER(val & mask | bits);
  }

  /// Sets output GPIO value to high.
  pub fn set_high(&self) {
    let bit: u32 = 1 << self.pin as uint;
    self.get_reg().set_BSRR(bit);
  }

  /// Sets output GPIO value to low.
  pub fn set_low(&self) {
    let bit: u32 = 1 << (self.pin as uint + 16);
    self.get_reg().set_BSRR(bit);
  }

  /// Returns input GPIO level.
  pub fn level(&self) -> ::hal::pin::GPIOLevel {
    let bit: u32 = 1 << (self.pin as uint);
    let reg = self.get_reg();

    match reg.IDR() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }

  fn get_reg(&self) -> &reg::GPIO {
    match self.port {
      PortA => &reg::GPIOA,
      PortB => &reg::GPIOB,
      PortC => &reg::GPIOC,
      PortD => &reg::GPIOD,
      PortE => &reg::GPIOE,
      PortF => &reg::GPIOF,
      PortG => &reg::GPIOG,
      PortH => &reg::GPIOH,
      PortI => &reg::GPIOI,
    }
  }
}

#[allow(dead_code)]
mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(GPIO: MODER, OTYPER, OSPEEDER, PUPDR, IDR, ODR, BSRR, LCKR, AFRL, AFRH)
  reg_rw!(GPIO, MODER,    set_MODER,    MODER)
  reg_rw!(GPIO, OTYPER,   set_OTYPER,   OTYPER)
  reg_rw!(GPIO, OSPEEDER, set_OSPEEDER, OSPEEDER)
  reg_rw!(GPIO, PUPDR,    set_PUPDR,    PUPDR)
  reg_rw!(GPIO, IDR,      set_IDR,      IDR)
  reg_rw!(GPIO, ODR,      set_ODR,      ODR)
  reg_rw!(GPIO, BSRR,     set_BSRR,     BSRR)
  reg_rw!(GPIO, LCKR,     set_LCKR,     LCKR)
  reg_rw!(GPIO, AFRL,     set_AFRL,     AFRL)
  reg_rw!(GPIO, AFRH,     set_AFRH,     AFRH)

  extern {
    #[link_name="iomem_GPIOA"] pub static GPIOA: GPIO;
    #[link_name="iomem_GPIOB"] pub static GPIOB: GPIO;
    #[link_name="iomem_GPIOC"] pub static GPIOC: GPIO;
    #[link_name="iomem_GPIOD"] pub static GPIOD: GPIO;
    #[link_name="iomem_GPIOE"] pub static GPIOE: GPIO;
    #[link_name="iomem_GPIOF"] pub static GPIOF: GPIO;
    #[link_name="iomem_GPIOG"] pub static GPIOG: GPIO;
    #[link_name="iomem_GPIOH"] pub static GPIOH: GPIO;
    #[link_name="iomem_GPIOI"] pub static GPIOI: GPIO;
    // define_reg!(GPIO_J: GPIO @ 0x40022400)
    // define_reg!(GPIO_K: GPIO @ 0x40022800)
  }
}
