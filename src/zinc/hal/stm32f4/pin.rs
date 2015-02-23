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

use self::Port::*;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

/// Available port names.
#[allow(missing_docs)]
#[derive(Copy)]
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
#[allow(missing_docs)]
#[derive(Copy)]
pub enum Function {
  GPIOIn      = 0,
  GPIOOut     = 1,
  AltFunction = 2,
  Analog      = 3,
}

impl Port {
  fn clock(self) -> peripheral_clock::PeripheralClock {
    use hal::stm32f4::peripheral_clock::PeripheralClock::*;
    match self {
      PortA => GPIOAClock,
      PortB => GPIOBClock,
      PortC => GPIOCClock,
      PortD => GPIODClock,
      PortE => GPIOEClock,
      PortF => GPIOFClock,
      PortG => GPIOGClock,
      PortH => GPIOHClock,
      PortI => GPIOIClock,
    }
  }
}

/// Pin configuration.
///
/// This structure shouldn't be used directly, pinmap.rs, available via pin::map
/// has all possible pin configurations.
#[derive(Copy)]
pub struct PinConf {
  /// Pin port, mcu-specific.
  pub port: Port,
  /// Pin number.
  pub pin: u8,
  /// Pin function, mcu-specific.
  pub function: Function,
}

impl PinConf {
  /// Setup the pin.
  #[inline(always)]
  pub fn setup(&self) {
    use self::Function::*;

    self.port.clock().enable();  // TODO(farcaller): should be done once per port

    let offset: u32 = self.pin as u32 * 2;
    let gpreg = self.get_reg();

    let bits: u32 = match self.function {
      GPIOOut => 0b01 << offset as usize,
      GPIOIn  => 0b00 << offset as usize,
      _       => unsafe { abort() },  // FIXME(farcaller): not implemented
    };
    let mask: u32 = !(0b11 << offset as usize);
    let val: u32 = gpreg.MODER();

    gpreg.set_MODER(val & mask | bits);
  }

  /// Sets output GPIO value to high.
  pub fn set_high(&self) {
    let bit: u32 = 1 << self.pin as usize;
    self.get_reg().set_BSRR(bit);
  }

  /// Sets output GPIO value to low.
  pub fn set_low(&self) {
    let bit: u32 = 1 << (self.pin as usize + 16);
    self.get_reg().set_BSRR(bit);
  }

  /// Returns input GPIO level.
  pub fn level(&self) -> ::hal::pin::GpioLevel {
    let bit: u32 = 1 << (self.pin as usize);
    let reg = self.get_reg();

    match reg.IDR() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }

  fn get_reg(&self) -> &reg::GPIO {
    match self.port {
      PortA => &reg::GPIO_A,
      PortB => &reg::GPIO_B,
      PortC => &reg::GPIO_C,
      PortD => &reg::GPIO_D,
      PortE => &reg::GPIO_E,
      PortF => &reg::GPIO_F,
      PortG => &reg::GPIO_G,
      PortH => &reg::GPIO_H,
      PortI => &reg::GPIO_I,
    }
  }
}

#[allow(dead_code)]
mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(GPIO: u32, MODER, OTYPER, OSPEEDER, PUPDR, IDR, ODR, BSRR, LCKR, AFRL, AFRH);
  reg_rw!(GPIO, u32, MODER,    set_MODER,    MODER);
  reg_rw!(GPIO, u32, OTYPER,   set_OTYPER,   OTYPER);
  reg_rw!(GPIO, u32, OSPEEDER, set_OSPEEDER, OSPEEDER);
  reg_rw!(GPIO, u32, PUPDR,    set_PUPDR,    PUPDR);
  reg_rw!(GPIO, u32, IDR,      set_IDR,      IDR);
  reg_rw!(GPIO, u32, ODR,      set_ODR,      ODR);
  reg_rw!(GPIO, u32, BSRR,     set_BSRR,     BSRR);
  reg_rw!(GPIO, u32, LCKR,     set_LCKR,     LCKR);
  reg_rw!(GPIO, u32, AFRL,     set_AFRL,     AFRL);
  reg_rw!(GPIO, u32, AFRH,     set_AFRH,     AFRH);

  extern {
    #[link_name="stm32f4_iomem_GPIOA"] pub static GPIO_A: GPIO;
    #[link_name="stm32f4_iomem_GPIOB"] pub static GPIO_B: GPIO;
    #[link_name="stm32f4_iomem_GPIOC"] pub static GPIO_C: GPIO;
    #[link_name="stm32f4_iomem_GPIOD"] pub static GPIO_D: GPIO;
    #[link_name="stm32f4_iomem_GPIOE"] pub static GPIO_E: GPIO;
    #[link_name="stm32f4_iomem_GPIOF"] pub static GPIO_F: GPIO;
    #[link_name="stm32f4_iomem_GPIOG"] pub static GPIO_G: GPIO;
    #[link_name="stm32f4_iomem_GPIOH"] pub static GPIO_H: GPIO;
    #[link_name="stm32f4_iomem_GPIOI"] pub static GPIO_I: GPIO;
    // define_reg!(GPIO_J: GPIO @ 0x40022400)
    // define_reg!(GPIO_K: GPIO @ 0x40022800)
  }
}
