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

//! GPIO configuration.

use hal::gpio::{Direction, In, Out, Level, Low, High};
use super::pin::{PinConf, Port0, Port1, Port2, Port3, Port4};

#[path="../../lib/ioreg.rs"] mod ioreg;

/// GPIO configuration.
pub struct GPIOConf {
  /// Pin configuration for this GPIO.
  pub pin: PinConf,

  /// Direction for GPIO, either `In` or `Out`.
  pub direction: Direction,
}

pub struct GPIO<'a> {
  pin: &'a PinConf,
}

impl GPIOConf {
  /// Returns a GPIO object that can be used to toggle or read GPIO value.
  pub fn setup<'a>(&'a self) -> GPIO<'a> {
    let gpio: GPIO = GPIO {
      pin: &self.pin,
    };

    gpio.set_direction(self.direction);

    gpio
  }
}

impl<'a> GPIO<'a> {
  /// Sets output GPIO value to high.
  pub fn set_high(&self) {
    self.reg().set_FIOSET(1 << self.pin.pin);
  }

  /// Sets output GPIO value to low.
  pub fn set_low(&self) {
    self.reg().set_FIOCLR(1 << self.pin.pin);
  }

  /// Sets output GPIO direction.
  pub fn set_direction(&self, new_mode: Direction) {
    let bit: u32 = 1 << self.pin.pin;
    let mask: u32 = !bit;
    let reg = self.reg();
    let val: u32 = reg.FIODIR();
    let new_val: u32 = match new_mode {
      In  => val & mask,
      Out => (val & mask) | bit,
    };

    reg.set_FIODIR(new_val);
  }

  /// Returns input GPIO level.
  pub fn level(&self) -> Level {
    let bit: u32 = 1 << self.pin.pin;
    let reg = self.reg();

    match reg.FIOPIN() & bit {
      0 => Low,
      _ => High,
    }
  }

  fn reg(&self) -> &reg::GPIO {
    match self.pin.port {
      Port0 => &reg::GPIO0,
      Port1 => &reg::GPIO1,
      Port2 => &reg::GPIO2,
      Port3 => &reg::GPIO3,
      Port4 => &reg::GPIO4,
    }
  }
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(GPIO: FIODIR, _r0, _r1, _r2, FIOMASK, FIOPIN, FIOSET, FIOCLR)
  reg_rw!(GPIO, FIODIR,  set_FIODIR,  FIODIR)
  reg_rw!(GPIO, FIOMASK, set_FIOMASK, FIOMASK)
  reg_rw!(GPIO, FIOPIN,  set_FIOPIN,  FIOPIN)
  reg_rw!(GPIO, FIOSET,  set_FIOSET,  FIOSET)
  reg_rw!(GPIO, FIOCLR,  set_FIOCLR,  FIOCLR)

  extern {
    #[link_name="iomem_GPIO0"] pub static GPIO0: GPIO;
    #[link_name="iomem_GPIO1"] pub static GPIO1: GPIO;
    #[link_name="iomem_GPIO2"] pub static GPIO2: GPIO;
    #[link_name="iomem_GPIO3"] pub static GPIO3: GPIO;
    #[link_name="iomem_GPIO4"] pub static GPIO4: GPIO;
  }
}
