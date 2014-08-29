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
use core::option::{Option};

#[path="../../lib/ioreg.rs"] mod ioreg;

/// Available port names.
#[allow(missing_doc)]
pub enum Port {
  Port0,
  Port1,
  Port2,
  Port3,
  Port4,
}

/// Pin functions (GPIO or up to three additional functions).
#[deriving(PartialEq)]
#[allow(missing_doc)]
pub enum Function {
  GPIO         = 0,
  AltFunction1 = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
}

/// Structure to describe the location of a pin
pub struct Pin {
  /// Port the pin is attached to
  port: Port,
  /// Pin number in the port
  pin: u8
}

impl Pin {
  /// Create and setup a Pin
  pub fn new(port: Port, pin_index: u8, function: Function,
      gpiodir: Option<::hal::pin::GPIODirection>) -> Pin {
    let pin = Pin {
      port: port,
      pin: pin_index,
    };
    pin.setup_regs(function, gpiodir);

    pin
  }

  fn setup_regs(&self, function: Function,
      gpiodir: Option<::hal::pin::GPIODirection>) {
    let (offset, reg) = self.get_pinsel_reg_and_offset();
    reg.set_pin(offset as uint, function as u32);

    if function == GPIO {
      (self as &::hal::pin::GPIO).set_direction(gpiodir.unwrap());
    }
  }

  fn gpioreg(&self) -> &'static reg::GPIO {
    match self.port {
      Port0 => &reg::GPIO0,
      Port1 => &reg::GPIO1,
      Port2 => &reg::GPIO2,
      Port3 => &reg::GPIO3,
      Port4 => &reg::GPIO4,
    }
  }

  fn get_pinsel_reg_and_offset(&self) -> (u8, &'static reg::PINSEL_pinsel) {
    match self.port {
      Port0 => match self.pin {
        0..15  => (self.pin,    &reg::PINSEL.pinsel[0]),
        16..30 => (self.pin-16, &reg::PINSEL.pinsel[1]),
        _      => unsafe { abort() },
      },
      Port1 => match self.pin {
        0..15  => (self.pin,    &reg::PINSEL.pinsel[2]),
        16..31 => (self.pin-16, &reg::PINSEL.pinsel[3]),
        _      => unsafe { abort() },
      },
      Port2 => match self.pin {
        0..13  => (self.pin,    &reg::PINSEL.pinsel[4]),
        _      => unsafe { abort() },
      },
      Port3 => match self.pin {
        25|26 => (self.pin-16,  &reg::PINSEL.pinsel[7]),
        _     => unsafe { abort() },
      },
      Port4 => match self.pin {
        28|29 => (self.pin-16,  &reg::PINSEL.pinsel[9]),
        _     => unsafe { abort() },
      },
    }
  }
}

impl ::hal::pin::GPIO for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpioreg().fioset.set_set(self.pin as uint, true);
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpioreg().fioclr.set_clr(self.pin as uint, true);
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GPIOLevel {
    let reg = self.gpioreg();
    match reg.fiopin.pin(self.pin as uint) {
      false => ::hal::pin::Low,
      _     => ::hal::pin::High,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, new_mode: ::hal::pin::GPIODirection) {
    let reg = self.gpioreg();
    let dir = match new_mode {
      ::hal::pin::In  => reg::INPUT,
      ::hal::pin::Out => reg::OUTPUT,
    };
    reg.fiodir.set_dir(self.pin as uint, dir);
  }
}

/// Sets the state of trace port interface.
pub fn set_trace_port_interface_enabled(enabled: bool) {
  reg::PINSEL.pinsel10.set_gpio_trace(enabled);
}

mod reg {
  use lib::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(PINSEL = {
    0x0      => reg32 pinsel[10] {     //! Pin function select register
      0..31    => pin[16]
    }

    0x28     => reg32 pinsel10 {       //! TPIU interface enable register
      3        => gpio_trace,
    }

    0x40     => reg32 pinmode[10] {    //! Pin pull-up/down select register
      0..31    => pin[16] {
        0x0    => PULL_UP,
        0x1    => REPEATER,
        0x2    => NO_PULL,
        0x3    => PULL_DOWN,
      }
    }

    0x68     => reg32 pinmode_od[5] {  //! Pin open-drain mode select register
      0..31    => pin[32],
    }

    0x7c     => reg32 i2cpadcfg {      //! I2C pin configuration register
      0        => sdadrv0,
      1        => sdai2c0,
      2        => scldrv0,
      3        => scli2c0,
    }
  })

  extern {
    #[link_name="lpc17xx_iomem_PINSEL"]  pub static PINSEL:  PINSEL;
  }

  ioregs!(GPIO = {
    0x0      => reg32 fiodir {
      0..31  => dir[32] {
        0x0    => INPUT,
        0x1    => OUTPUT,
      }
    }

    0x10     => reg32 fiomask {
      0..31  => mask[32],
    }

    0x14     => reg32 fiopin {
      0..31  => pin[32],
    }

    0x18     => reg32 fioset {
      0..31  => set[32]: wo,
    }

    0x1c     => reg32 fioclr {
      0..31  => clr[32]: wo,
    }
  })

  extern {
    #[link_name="lpc17xx_iomem_GPIO0"] pub static GPIO0: GPIO;
    #[link_name="lpc17xx_iomem_GPIO1"] pub static GPIO1: GPIO;
    #[link_name="lpc17xx_iomem_GPIO2"] pub static GPIO2: GPIO;
    #[link_name="lpc17xx_iomem_GPIO3"] pub static GPIO3: GPIO;
    #[link_name="lpc17xx_iomem_GPIO4"] pub static GPIO4: GPIO;
  }
}
