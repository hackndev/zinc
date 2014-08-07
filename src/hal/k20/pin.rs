// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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
Pin configuration.

Some pins that could be configured here may be missing from actual MCU depending
on the package.
*/

use core::option::Option;

#[path="../../lib/ioreg.rs"] mod ioreg;

/// A GPIO bank
#[allow(missing_doc)]
pub struct GPIO {
  gpio: &'static reg::GPIO,
  port: &'static reg::PORT,
}

impl GPIO {
  /// Returns a GPIO bank instance.
  pub fn new(gpio: &'static reg::GPIO, port: &'static reg::PORT) -> GPIO {
    GPIO { gpio: gpio, port: port }
  }

  /// Returns a Pin to control the the requested pin.
  pub fn get_pin(&self, pin: u8,
                 gpiodir: Option<::hal::pin::GPIODirection>) -> Pin {
    Pin::new(self.gpio, self.port, pin, Gpio, gpiodir)
  }
}

/// A pin.
#[allow(missing_doc)]
pub struct Pin {
  gpio: &'static reg::GPIO,
  pin: u8,
}

/// Pin functions (GPIO or up to seven additional functions).
#[deriving(PartialEq)]
#[allow(missing_doc)]
pub enum Function {
  Analog       = 0,
  Gpio        = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
  AltFunction4 = 4,
  AltFunction5 = 5,
  AltFunction6 = 6,
  AltFunction7 = 7,
}

/// Pull-up/-down configuration.
#[allow(missing_doc)]
pub enum PullConf {
  PullNone   = 0,
  PullUp     = 1,
  PullDown   = 2,
}

/// Pin output driver strength.
#[allow(missing_doc)]
pub enum DriveStrength {
  DriveStrengthHigh   = 0,
  DriveStrengthLow    = 1,
}

/// Pin output drive slew rate.
#[allow(missing_doc)]
pub enum SlewRate {
  SlewFast   = 0,
  SlewSlow   = 1,
}

impl Pin {
  /// Create and setup a Pin.
  pub fn new(gpio: &'static reg::GPIO, port: &'static reg::PORT,
             pin_index: u8, function: Function, 
             gpiodir: Option<::hal::pin::GPIODirection>) -> Pin {
    let pin = Pin {
      gpio: gpio,
      pin: pin_index,
    };
    pin.setup_regs(port, function, gpiodir, PullNone,
                   DriveStrengthHigh, SlewSlow, false, false);

    pin
  }

  fn setup_regs(&self, port: &'static reg::PORT, function: Function,
      gpiodir: Option<::hal::pin::GPIODirection>,
      pull: PullConf, drive_strength: DriveStrength,
      slew_rate: SlewRate, filter: bool, open_drain: bool) {

    let value =
          (pull as u32 << 0)
          | (slew_rate as u32 << 2)
          | (filter as u32 << 4)
          | (open_drain as u32 << 5)
          | (drive_strength as u32 << 6)
          | (function as u32 << 8);
    port.PCR[self.pin as uint].set(value);

    if function == Gpio {
      (self as &::hal::pin::GPIO).set_direction(gpiodir.unwrap());
    }
  }

  fn gpiobit(&self) -> u32 {
    1 << (self.pin as uint)
  }
}

impl ::hal::pin::GPIO for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpio.set_PSOR(self.gpiobit());
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpio.set_PCOR(self.gpiobit());
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GPIOLevel {
    let bit: u32 = self.gpiobit();
    let reg = self.gpio;

    match reg.PDIR() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, new_mode: ::hal::pin::GPIODirection) {
    let bit: u32 = self.gpiobit();
    let reg = self.gpio;
    let val: u32 = reg.PDDR();
    let new_val: u32 = match new_mode {
      ::hal::pin::In  => val & !bit,
      ::hal::pin::Out => val | bit,
    };

    reg.set_PDDR(new_val);
  }
}

#[allow(missing_doc)]
pub mod reg {
  use lib::volatile_cell::VolatileCell;

  #[allow(uppercase_variables)]
  pub struct PORT {
    pub PCR: [VolatileCell<u32>, ..32],
    pub GPCLR: VolatileCell<u32>,
    pub GPCHR: VolatileCell<u32>,
    pub ISFR: VolatileCell<u32>,
    pub DFER: VolatileCell<u32>,
    pub DFCR: VolatileCell<u32>,
    pub DFWR: VolatileCell<u32>,
  }

  ioreg!(GPIO: u32, PDOR, PSOR, PCOR, PTOR, PDIR, PDDR)
  reg_rw!(GPIO, u32, PDOR,  set_PDOR,  PDOR)
  reg_rw!(GPIO, u32, PSOR,  set_PSOR,  PSOR)
  reg_rw!(GPIO, u32, PCOR,  set_PCOR,  PCOR)
  reg_rw!(GPIO, u32, PTOR,  set_PTOR,  PTOR)
  reg_rw!(GPIO, u32, PDIR,  set_PDIR,  PDIR)
  reg_rw!(GPIO, u32, PDDR,  set_PDDR,  PDDR)
}
