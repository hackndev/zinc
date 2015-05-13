// Zinc, the bare metal stack for rust.
// Copyright 2014 Lionel Flandrin <lionel@svkt.org>
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

//! Pin configuration
//! Allows GPIO configuration
//! Pin muxing not implemented yet.

use hal::pin::{Gpio, GpioDirection, In, Out, GpioLevel, High, Low};
use hal::tiva_c::sysctl;
use util::support::get_reg_ref;

/// The pins are accessed through ports. Each port has 8 pins and are identified
/// by a letter (PortA, PortB, etc...).
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum PortId {
  PortA,
  PortB,
  PortC,
  PortD,
  PortE,
  PortF,
}

/// Structure describing a single HW pin
#[derive(Clone, Copy)]
pub struct Pin {
  /// Timer register interface
  regs: &'static reg::Port,
  /// Pin index in the port
  index: usize,
}

impl Pin {
  /// Create and configure a Pin
  pub fn new(pid:       PortId,
             pin_index: u8,
             dir:       GpioDirection,
             function:  u8) -> Pin {

    // Retrieve GPIO port peripheral to enable it
    let (periph, regs) = match pid {
      PortId::PortA => (sysctl::periph::gpio::PORT_A, reg::PORT_A),
      PortId::PortB => (sysctl::periph::gpio::PORT_B, reg::PORT_B),
      PortId::PortC => (sysctl::periph::gpio::PORT_C, reg::PORT_C),
      PortId::PortD => (sysctl::periph::gpio::PORT_D, reg::PORT_D),
      PortId::PortE => (sysctl::periph::gpio::PORT_E, reg::PORT_E),
      PortId::PortF => (sysctl::periph::gpio::PORT_F, reg::PORT_F),
    };

    periph.ensure_enabled();

    let pin = Pin { regs: get_reg_ref(regs), index: pin_index as usize };

    pin.configure(dir, function);

    pin
  }

  /// Configure GPIO pin
  fn configure(&self, dir: GpioDirection, function: u8) {
    // Disable the GPIO during reconfig
    self.regs.den.set_den(self.index, false);

    self.set_direction(dir);

    // Configure the "alternate function". AFSEL 0 means GPIO, 1 means the port
    // is driven by another peripheral. When AFSEL is 1 the actual function
    // config goes into the CTL register.
    match function {
      0 => {
        self.regs.afsel.set_afsel(self.index,
                                  reg::Port_afsel_afsel::GPIO);
      },
      f => {
        self.regs.afsel.set_afsel(self.index,
                                  reg::Port_afsel_afsel::PERIPHERAL);

        self.regs.pctl.set_pctl(self.index, f as u32);
      }
    }

    // We can chose to drive each GPIO at either 2, 4 or 8mA. Default to 2mA for
    // now.
    // TODO(simias): make that configurable
    self.regs.dr2r.set_dr2r(self.index, true);
    self.regs.dr4r.set_dr4r(self.index, false);
    self.regs.dr8r.set_dr8r(self.index, false);

    // TODO(simias): configure open drain/pull up/pull down/slew rate if necessary

    self.regs.odr.set_odr(self.index, false);
    self.regs.pur.set_pur(self.index, false);
    self.regs.pdr.set_pdr(self.index, false);

    // Enable GPIO
    self.regs.den.set_den(self.index, true);
  }

  fn set_level(&self, level: bool) {
    self.regs.data.set_data(self.index, level);
  }
}

impl Gpio for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.set_level(true);
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.set_level(false);
  }

  /// Returns input GPIO level.
  fn level(&self) -> GpioLevel {
    match self.regs.data.data(self.index) {
      true  => High,
      false => Low,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, dir: GpioDirection) {
    self.regs.dir.set_dir(self.index,
                          match dir {
                            In  => reg::Port_dir_dir::INPUT,
                            Out => reg::Port_dir_dir::OUTPUT,
                          });
  }
}

pub mod reg {
  //! Pin registers definition
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(Port = {
    0x3FC => reg32 data {
      //! Pin value
      0..7 => data[8]
    }

    0x400 => reg32 dir {
      //! Pin direction
      0..7 => dir[8] {
        0 => INPUT,
        1 => OUTPUT,
      }
    }

    0x420 => reg32 afsel {
      //! Pin alternate function
      0..7 => afsel[8] {
        0 => GPIO,
        1 => PERIPHERAL,
      }
    }

    0x500 => reg32 dr2r {
      //! Select 2mA drive strength
      0..7 => dr2r[8]
    }

    0x504 => reg32 dr4r {
      //! Select 4mA drive strength
      0..7 => dr4r[8]
    }

    0x508 => reg32 dr8r {
      //! Select 8mA drive strength
      0..7 => dr8r[8]
    }

    0x50C => reg32 odr {
      //! Configure pin as open drain
      0..7 => odr[8]
    }

    0x510 => reg32 pur {
      //! Enable pin pull-up
      0..7 => pur[8]
    }

    0x514 => reg32 pdr {
      //! Enable pin pull-down
      0..7 => pdr[8]
    }

    0x518 => reg32 slr {
      //! Slew rate control enable (only available for 8mA drive strength)
      0..7 => slr[8]
    }

    0x51C => reg32 den {
      //! Enable pin
      0..7 => den[8]
    }

    0x52C => reg32 pctl {
      //! Pin function selection when afsel is set for the pin.
      0..31 => pctl[8]
    }
  });

  pub const PORT_A: *const Port = 0x40004000 as *const Port;
  pub const PORT_B: *const Port = 0x40005000 as *const Port;
  pub const PORT_C: *const Port = 0x40006000 as *const Port;
  pub const PORT_D: *const Port = 0x40007000 as *const Port;
  pub const PORT_E: *const Port = 0x40024000 as *const Port;
  pub const PORT_F: *const Port = 0x40025000 as *const Port;
}
