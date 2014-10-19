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

//! Pin configuration for ST STM32L1.
//!
//! Some pins that could be configured here may be missing from actual MCU
//! depending on the package.

use super::peripheral_clock;
use core::intrinsics::abort;

#[path="../../util/ioreg.rs"] mod ioreg;

/// Available port names.
#[allow(missing_doc)]
#[repr(u8)]
pub enum Port {
  PortA,
  PortB,
  PortC,
  PortD,
  PortE,
  PortF,
  PortG,
  PortH,
}

/// Pin output type.
#[allow(missing_doc)]
#[repr(u8)]
pub enum OutputType {
  OutPushPull  = 0,
  OutOpenDrain = 1,
}

/// Pin pull resistors: up, down, or none.
#[allow(missing_doc)]
#[repr(u8)]
pub enum PullType {
  PullNone = 0,
  PullUp   = 1,
  PullDown = 2,
}

/// Pin speed.
#[repr(u8)]
pub enum Speed {
  /// 400 KHz
  VeryLow = 0,
  /// 2 MHz
  Low     = 1,
  /// 10 MHz
  Medium  = 2,
  /// 40 MHz
  High    = 3,
}

/// Pin mode.
pub enum Mode {
  /// GPIO Input Mode
  GpioIn,
  /// GPIO Output Mode
  GpioOut(OutputType, Speed),
  // GPIO Alternate function Mode
  //AltFunction(OutputType, Speed),
  // GPIO Analog Mode
  //Analog,
}

impl Port {
  fn clock(self) -> peripheral_clock::PeripheralClock {
    peripheral_clock::ClockAhb(match self {
      PortA => peripheral_clock::GpioA,
      PortB => peripheral_clock::GpioB,
      PortC => peripheral_clock::GpioC,
      PortD => peripheral_clock::GpioD,
      PortE => peripheral_clock::GpioE,
      PortF => peripheral_clock::GpioF,
      PortG => peripheral_clock::GpioG,
      PortH => peripheral_clock::GpioH,
    })
  }
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
  /// Pin mode, mcu-specific.
  pub mode: Mode,
  /// Pin pull type.
  pub pull_type: PullType,
}

impl PinConf {
  /// Setup the pin.
  #[no_stack_check]
  #[inline(always)]
  pub fn setup(&self) {
    self.port.clock().enable();  // TODO(farcaller): should be done once per port

    let offset1 = self.pin as uint;
    let mask1 = !(0b1 << offset1);
    let offset2 = self.pin as uint * 2;
    let mask2: u32 = !(0b11 << offset2);
    let gpreg = self.get_reg();

    let fun: u32 = match self.mode {
      GpioIn  => 0b00,
      GpioOut(otype, speed) => {
          let tv: u32 = gpreg.OTYPER() & mask1;
          gpreg.set_OTYPER(tv | (otype as u32 << offset1));
          let sv: u32 = gpreg.OSPEEDR() & mask2;
          gpreg.set_OSPEEDR(sv | (speed as u32 << offset2));
          0b01
      },
      /*AltFunction(_, _) => {
          unsafe { abort() } //TODO
          0b10
      },
      Analog => {
          unsafe { abort() } //TODO
          0b11
      },*/
    };

    let mode: u32 = gpreg.MODER() & mask2;
    gpreg.set_MODER(mode | (fun << offset2));

    let pull: u32 = gpreg.PUPDR() & mask2;
    let pull_val = (self.pull_type as u32) << offset2;
    gpreg.set_PUPDR(pull | pull_val);
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
    }
  }
}

#[allow(dead_code)]
mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(GPIO: u32, MODER, OTYPER, OSPEEDR, PUPDR, IDR, ODR, BSRR, LCKR, AFRL, AFRH, BRR)
  reg_rw!(GPIO, u32, MODER,    set_MODER,    MODER)     // port mode
  reg_rw!(GPIO, u32, OTYPER,   set_OTYPER,   OTYPER)    // port output type
  reg_rw!(GPIO, u32, OSPEEDR,  set_OSPEEDR,  OSPEEDR)   // port output speed
  reg_rw!(GPIO, u32, PUPDR,    set_PUPDR,    PUPDR)     // port pull-up/pull-down
  reg_rw!(GPIO, u32, IDR,      set_IDR,      IDR)       // port input data
  reg_rw!(GPIO, u32, ODR,      set_ODR,      ODR)       // port output data
  reg_rw!(GPIO, u32, BSRR,     set_BSRR,     BSRR)      // port bit set/reset
  reg_rw!(GPIO, u32, LCKR,     set_LCKR,     LCKR)      // port configuration lock
  reg_rw!(GPIO, u32, AFRL,     set_AFRL,     AFRL)      // alternate function low
  reg_rw!(GPIO, u32, AFRH,     set_AFRH,     AFRH)      // alternate function high
  reg_rw!(GPIO, u32, BRR,      set_BRR,      BRR)       // bit reset register

  extern {
    #[link_name="stm32l1_iomem_GPIOA"] pub static GPIOA: GPIO;
    #[link_name="stm32l1_iomem_GPIOB"] pub static GPIOB: GPIO;
    #[link_name="stm32l1_iomem_GPIOC"] pub static GPIOC: GPIO;
    #[link_name="stm32l1_iomem_GPIOD"] pub static GPIOD: GPIO;
    #[link_name="stm32l1_iomem_GPIOE"] pub static GPIOE: GPIO;
    #[link_name="stm32l1_iomem_GPIOF"] pub static GPIOF: GPIO;
    #[link_name="stm32l1_iomem_GPIOG"] pub static GPIOG: GPIO;
    #[link_name="stm32l1_iomem_GPIOH"] pub static GPIOH: GPIO;
  }
}
