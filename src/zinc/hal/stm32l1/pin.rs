// Zinc, the bare metal stack for rust.
// Copyright 2014 Dzmitry "kvark" Malyshau <kvarkus@gmail.com>
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
  //TODO (kvark): implement this
  // GPIO Alternate function Mode
  //AltFunction(OutputType, Speed),
  // GPIO Analog Mode
  //Analog,
}

/// Pin configuration.
pub struct Pin {
  /// Pin index.
  pub index: u8,
  /// GPIO register
  reg: &'static reg::GPIO,
}

impl Pin {
  /// Setup the pin.
  #[inline(always)]
  pub fn new(port: Port, pin_index: u8, mode: Mode, pull_type: PullType) -> Pin {
    let (reg, clock) = match port {
      PortA => (&reg::GPIOA, peripheral_clock::GpioA),
      PortB => (&reg::GPIOB, peripheral_clock::GpioB),
      PortC => (&reg::GPIOC, peripheral_clock::GpioC),
      PortD => (&reg::GPIOD, peripheral_clock::GpioD),
      PortE => (&reg::GPIOE, peripheral_clock::GpioE),
      PortF => (&reg::GPIOF, peripheral_clock::GpioF),
      PortG => (&reg::GPIOG, peripheral_clock::GpioG),
      PortH => (&reg::GPIOH, peripheral_clock::GpioH),
    };
    // TODO(farcaller): should be done once per port
    peripheral_clock::ClockAhb(clock).enable();

    let offset1 = pin_index as uint;
    let mask1 = !(0b1u16 << offset1);
    let offset2 = pin_index as uint * 2;
    let mask2: u32 = !(0b11 << offset2);

    let fun: u32 = match mode {
      GpioIn  => 0b00,
      GpioOut(otype, speed) => {
          let tv: u16 = reg.otyper.otype() & mask1;
          reg.otyper.set_otype(tv | (otype as u16 << offset1));
          let sv: u32 = reg.ospeedr.speed() & mask2;
          reg.ospeedr.set_speed(sv | (speed as u32 << offset2));
          0b01
      },
      /*TODO (kvark): implement these modes
      AltFunction(_, _) => {
          unsafe { abort() } //TODO
          0b10
      },
      Analog => {
          unsafe { abort() } //TODO
          0b11
      },*/
    };

    let mode: u32 = reg.moder.mode() & mask2;
    reg.moder.set_mode(mode | (fun << offset2));

    let pull: u32 = reg.pupdr.mode() & mask2;
    let pull_val = (pull_type as u32) << offset2;
    reg.pupdr.set_mode(pull | pull_val);

    Pin {
      index: pin_index,
      reg: reg,
    }
  }
}

impl ::hal::pin::GPIO for Pin {
  fn set_high(&self) {
    let bit: u32 = 1 << self.index as uint;
    self.reg.bsrr.set_reset(bit);
  }

  fn set_low(&self) {
    let bit: u32 = 1 << (self.index as uint + 16);
    self.reg.bsrr.set_reset(bit);
  }

  fn level(&self) -> ::hal::pin::GPIOLevel {
    let bit = 1u16 << (self.index as uint);

    match self.reg.idr.input() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }

  fn set_direction(&self, _new_mode: ::hal::pin::GPIODirection) {
    //TODO(kvark)
    unsafe { abort() }
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(GPIO = {
    0x00 => reg32 moder {   // port mode
      31..0 => mode : rw,
    },
    0x04 => reg16 otyper {  // port output type
      15..0 => otype : rw,
    },
    0x08 => reg32 ospeedr { // port output speed
      31..0 => speed : rw,
    },
    0x0C => reg32 pupdr {   // port pull-up/pull-down
      31..0 => mode : rw,
    },
    0x10 => reg16 idr {     // port input data
      15..0 => input : rw,
    },
    0x14 => reg16 odr {     // port output data
      15..0 => output : rw,
    },
    0x18 => reg32 bsrr {    // port bit set/reset
      31..0 => reset : rw,
    },
    0x1C => reg32 lckr {    // port configuration lock
      31..0 => config_lock : rw,
    },
    0x20 => reg32 afrl {    // alternate function low
      31..0 => alt_fun : rw,
    },
    0x24 => reg32 afrh {     // alternate function high
      31..0 => alt_fun : rw,
    },
    0x28 => reg16 brr {      // bit reset register
      15..0 => reset : rw,
    },
  })

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
