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
use self::Port::*;

/// Available port names.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Port {
  PortA,
  PortB,
  PortC,
  PortD,
  PortE,
  PortF,
  PortG,
}

/// Pin output mode.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PinConf {
  /* Input mode */
  InAnalog,
  InFloating,
  InPullUpDown,
  /* Output mode, max speed 10 MHz */
  OutPushPull_10MHz,
  OutOpenDrain_10MHz,
  OutPushPullAlt_10MHz,
  OutOpenDrainAlt_10MHz,
  /* Output mode, max speed 2 MHz */
  OutPushPull_2MHz,
  OutOpenDrain_2MHz,
  OutPushPullAlt_2MHz,
  OutOpenDrainAlt_2MHz,
  /* Output mode, max speed 50 MHz */
  OutPushPull_50MHz,
  OutOpenDrain_50MHz,
  OutPushPullAlt_50MHz,
  OutOpenDrainAlt_50MHz,
}

/// Pin configuration.
#[derive(Clone, Copy)]
pub struct Pin {
  /// Pin index.
  pub index: u8,
  /// GPIO register
  reg: &'static reg::GPIO,
}

impl Pin {
  /// Setup the pin.
  #[inline(always)]
  pub fn new(port: Port, pin_index: u8, mode: PinConf) -> Pin {
    use hal::stm32l1::peripheral_clock::BusAhb as clock;
    use self::PinConf::*;
    let (reg, clock) = match port {
      PortA => (&reg::GPIOA, clock::GpioA),
      PortB => (&reg::GPIOB, clock::GpioB),
      PortC => (&reg::GPIOC, clock::GpioC),
      PortD => (&reg::GPIOD, clock::GpioD),
      PortE => (&reg::GPIOE, clock::GpioE),
      PortF => (&reg::GPIOF, clock::GpioF),
      PortG => (&reg::GPIOG, clock::GpioG),
    };
    // TODO(farcaller): should be done once per port
    peripheral_clock::PeripheralClock::Apb2(clock).enable();

    let conf: u32 = match mode {
      /* Input mode */
      InAnalog              => 0b00_00,
      InFloating            => 0b01_00,
      InPullUpDown          => 0b10_00,
      /* Output mode, max speed 10 MHz */
      OutPushPull_10MHz     => 0b00_01,
      OutOpenDrain_10MHz    => 0b01_01,
      OutPushPullAlt_10MHz  => 0b10_01,
      OutOpenDrainAlt_10MHz => 0b11_01,
      /* Output mode, max speed 2 MHz */
      OutPushPull_2MHz      => 0b00_10,
      OutOpenDrain_2MHz     => 0b01_10,
      OutPushPullAlt_2MHz   => 0b10_10,
      OutOpenDrainAlt_2MHz  => 0b11_10,
      /* Output mode, max speed 50 MHz */
      OutPushPull_50MHz     => 0b00_11,
      OutOpenDrain_50MHz    => 0b01_11,
      OutPushPullAlt_50MHz  => 0b10_11,
      OutOpenDrainAlt_50MHz => 0b11_11,
    };

    let offset = (pin_index % 8) as usize * 4;
    let mask = !(0xFu32 << offset);

    if pin_index < 8 {
        let mode: u32 = reg.crlr.mode() & mask;
        reg.moder.set_crlr(conf << offset);
    } else {
        let mode: u32 = reg.crhr.mode() & mask;
        reg.moder.set_crhr(conf << offset);
    }

    Pin {
      index: pin_index,
      reg: reg,
    }
  }
}

impl ::hal::pin::Gpio for Pin {
  fn set_high(&self) {
    let bit: u32 = 1 << self.index as usize;
    self.reg.bsrr.set_reset(bit);
  }

  fn set_low(&self) {
    let bit: u32 = 1 << (self.index as usize + 16);
    self.reg.bsrr.set_reset(bit);
  }

  fn level(&self) -> ::hal::pin::GpioLevel {
    let bit = 1u16 << (self.index as usize);

    match self.reg.idr.input() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }
}

mod reg {
  use volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(GPIO = {
    0x00 => reg32 crlr {   // port mode
      31..0 => crl : rw,
    },
    0x04 => reg32 crhr {  // port output type
      31..0 => crh : rw,
    },
    0x08 => reg16 idr {     // port input data
      15..0 => input : rw,
    },
    0x0c => reg16 odr {     // port output data
      15..0 => output : rw,
    },
    0x10 => reg32 bsrr {    // port bit set/reset
      31..0 => reset : rw,
    },
    // 0x14 => reg16 brr {      // bit reset register
    //   15..0 => reset : rw,
    // },
    0x18 => reg32 lckr {    // port configuration lock
      31..0 => config_lock : rw,
    },
  });

  extern {
    #[link_name="stm32l1_iomem_GPIOA"] pub static GPIOA: GPIO;
    #[link_name="stm32l1_iomem_GPIOB"] pub static GPIOB: GPIO;
    #[link_name="stm32l1_iomem_GPIOC"] pub static GPIOC: GPIO;
    #[link_name="stm32l1_iomem_GPIOD"] pub static GPIOD: GPIO;
    #[link_name="stm32l1_iomem_GPIOE"] pub static GPIOE: GPIO;
    #[link_name="stm32l1_iomem_GPIOF"] pub static GPIOF: GPIO;
    #[link_name="stm32l1_iomem_GPIOG"] pub static GPIOG: GPIO;
  }
}
