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

#[path="../../lib/ioreg.rs"] mod ioreg;

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

/// Available port names.
pub enum Port {
  Port0,
  Port1,
  Port2,
  Port3,
  Port4,
}

/// Pin functions (GPIO or up to three additional functions).
pub enum Function {
  GPIO         = 0,
  AltFunction1 = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
}

impl PinConf {
  #[no_split_stack]
  #[inline(always)]
  pub fn setup(&self) {
    let (offset, reg) = self.get_pinsel_reg_and_offset();

    let fun_bits: u32  = self.function as u32 << (offset as uint * 2);
    let mask_bits: u32 = !(3u32 << (offset as uint * 2));

    let val: u32 = reg.value();
    let new_val = (val & mask_bits) | fun_bits;
    reg.set_value(new_val);
  }

  #[no_split_stack]
  #[inline(always)]
  fn get_pinsel_reg_and_offset(&self) -> (u8, &reg::PINSEL) {
    match self.port {
      Port0 => match self.pin {
        0..15  => (self.pin,    &reg::PINSEL0),
        16..30 => (self.pin-16, &reg::PINSEL1),
        _      => unsafe { abort() },
      },
      Port1 => match self.pin {
        0..15  => (self.pin,    &reg::PINSEL2),
        16..31 => (self.pin-16, &reg::PINSEL3),
        _      => unsafe { abort() },
      },
      Port2 => match self.pin {
        0..13  => (self.pin,    &reg::PINSEL4),
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

#[no_split_stack]
#[inline(always)]
/// Sets the state of trace port interface.
pub fn set_trace_port_interface_enabled(enabled: bool) {
  let value: u32 = if enabled { 0b1000 } else { 0 };
  reg::PINSEL10.set_value(value);
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(PINSEL: value)
  reg_rw!(PINSEL, value, set_value, value)

  extern {
    #[link_name="iomem_PINSEL0"]  pub static PINSEL0:  PINSEL;
    #[link_name="iomem_PINSEL1"]  pub static PINSEL1:  PINSEL;
    #[link_name="iomem_PINSEL2"]  pub static PINSEL2:  PINSEL;
    #[link_name="iomem_PINSEL3"]  pub static PINSEL3:  PINSEL;
    #[link_name="iomem_PINSEL4"]  pub static PINSEL4:  PINSEL;
    #[link_name="iomem_PINSEL7"]  pub static PINSEL7:  PINSEL;
    #[link_name="iomem_PINSEL9"]  pub static PINSEL9:  PINSEL;
    #[link_name="iomem_PINSEL10"] pub static PINSEL10: PINSEL;
  }
}
