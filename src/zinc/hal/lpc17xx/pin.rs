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
Pin configuration.

Some pins that could be configured here may be missing from actual MCU depending
on the package.
*/

use core::intrinsics::abort;
use core::option::Option;
use core::marker::Copy;

use self::Port::*;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

/// Available port names.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum Port {
  Port0,
  Port1,
  Port2,
  Port3,
  Port4,
}

/// Pin functions (GPIO or up to three additional functions).
#[derive(PartialEq)]
#[allow(missing_docs)]
pub enum Function {
  Gpio         = 0,
  AltFunction1 = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
}

impl Copy for Function {}

/// Structure to describe the location of a pin
#[derive(Copy)]
pub struct Pin {
  /// Port the pin is attached to
  port: Port,
  /// Pin number in the port
  pin: u8
}

impl Pin {
  /// Create and setup a Pin
  pub fn new(port: Port, pin_index: u8, function: Function,
      gpiodir: Option<::hal::pin::GpioDirection>) -> Pin {
    let pin = Pin {
      port: port,
      pin: pin_index,
    };

    pin.setup_regs(function, gpiodir);

    pin
  }

  fn setup_regs(&self, function: Function,
      gpiodir: Option<::hal::pin::GpioDirection>) {
    let (offset, reg) = self.get_pinsel_reg_and_offset();

    let fun_bits: u32  = (function as u32) << ((offset as usize) * 2);
    let mask_bits: u32 = !(3u32 << ((offset as usize) * 2));

    let val: u32 = reg.value();
    let new_val = (val & mask_bits) | fun_bits;
    reg.set_value(new_val);

    if function == Function::Gpio {
      (self as &::hal::pin::Gpio).set_direction(gpiodir.unwrap());
    }
  }

  fn gpioreg(&self) -> &reg::Gpio {
    match self.port {
      Port0 => &reg::GPIO_0,
      Port1 => &reg::GPIO_1,
      Port2 => &reg::GPIO_2,
      Port3 => &reg::GPIO_3,
      Port4 => &reg::GPIO_4,
    }
  }

  fn get_pinsel_reg_and_offset(&self) -> (u8, &reg::PINSEL) {
    match self.port {
      Port0 => match self.pin {
        0...15  => (self.pin,    &reg::PINSEL0),
        16...30 => (self.pin-16, &reg::PINSEL1),
        _      => unsafe { abort() },
      },
      Port1 => match self.pin {
        0...15  => (self.pin,    &reg::PINSEL2),
        16...31 => (self.pin-16, &reg::PINSEL3),
        _      => unsafe { abort() },
      },
      Port2 => match self.pin {
        0...13  => (self.pin,    &reg::PINSEL4),
        _      => unsafe { abort() },
      },
      Port3 => match self.pin {
        25|26 => (self.pin-16,  &reg::PINSEL7),
        _     => unsafe { abort() },
      },
      Port4 => match self.pin {
        28|29 => (self.pin-16,  &reg::PINSEL9),
        _     => unsafe { abort() },
      },
    }
  }
}

impl ::hal::pin::Gpio for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpioreg().set_FIOSET(1 << (self.pin as usize));
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpioreg().set_FIOCLR(1 << (self.pin as usize));
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GpioLevel {
    let bit: u32 = 1 << (self.pin as usize);
    let reg = self.gpioreg();

    match reg.FIOPIN() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, new_mode: ::hal::pin::GpioDirection) {
    let bit: u32 = 1 << (self.pin as usize);
    let mask: u32 = !bit;
    let reg = self.gpioreg();
    let val: u32 = reg.FIODIR();
    let new_val: u32 = match new_mode {
      ::hal::pin::In  => val & mask,
      ::hal::pin::Out => (val & mask) | bit,
    };

    reg.set_FIODIR(new_val);
  }
}

/// Sets the state of trace port interface.
pub fn set_trace_port_interface_enabled(enabled: bool) {
  let value: u32 = if enabled { 0b1000 } else { 0 };
  reg::PINSEL10.set_value(value);
}

mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(PINSEL: u32, value);
  reg_rw!(PINSEL, u32, value, set_value, value);

  extern {
    #[link_name="lpc17xx_iomem_PINSEL0"]  pub static PINSEL0:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL1"]  pub static PINSEL1:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL2"]  pub static PINSEL2:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL3"]  pub static PINSEL3:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL4"]  pub static PINSEL4:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL7"]  pub static PINSEL7:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL9"]  pub static PINSEL9:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL10"] pub static PINSEL10: PINSEL;
  }

  ioreg_old!(Gpio: u32, FIODIR, _r0, _r1, _r2, FIOMASK, FIOPIN, FIOSET, FIOCLR);
  reg_rw!(Gpio, u32, FIODIR,  set_FIODIR,  FIODIR);
  reg_rw!(Gpio, u32, FIOMASK, set_FIOMASK, FIOMASK);
  reg_rw!(Gpio, u32, FIOPIN,  set_FIOPIN,  FIOPIN);
  reg_rw!(Gpio, u32, FIOSET,  set_FIOSET,  FIOSET);
  reg_rw!(Gpio, u32, FIOCLR,  set_FIOCLR,  FIOCLR);

  extern {
    #[link_name="lpc17xx_iomem_GPIO0"] pub static GPIO_0: Gpio;
    #[link_name="lpc17xx_iomem_GPIO1"] pub static GPIO_1: Gpio;
    #[link_name="lpc17xx_iomem_GPIO2"] pub static GPIO_2: Gpio;
    #[link_name="lpc17xx_iomem_GPIO3"] pub static GPIO_3: Gpio;
    #[link_name="lpc17xx_iomem_GPIO4"] pub static GPIO_4: Gpio;
  }
}
