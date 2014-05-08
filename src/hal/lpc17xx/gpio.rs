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

use std::option::{Option, None};
use std::intrinsics::{abort, transmute};

use hal::gpio::{Direction, In, Out, Level, Low, High};
use hal::gpio::{GPIOISRHandler, InterruptEdge, Rising, Falling};
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

  pub fn set_interrupt_handler(&self, edge: InterruptEdge,
      h: Option<&GPIOISRHandler>) {
    let b = match self.pin.port {
      Port0 => match edge {
        Rising  => GPIO0Rising,
        Falling => GPIO0Falling,
      },
      Port2 => match edge {
        Rising  => GPIO2Rising,
        Falling => GPIO2Falling,
      },
      _ => unsafe { abort() },
    };
    set_gpio_handler(b, self.pin.pin, h)
  }

  #[inline(always)]
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

type GPIOBank = [Option<&'static GPIOISRHandler>, ..32];

// TODO(farcaller): those static muts are accessed from both userland and isr,
// so they must be synchronized in some way. Apparently, no scheduling should
// happen while in gpio isr.
static mut GPIO0RisingBank: GPIOBank = [None, ..32];
static mut GPIO0FallingBank: GPIOBank = [None, ..32];
static mut GPIO2RisingBank: GPIOBank = [None, ..32];
static mut GPIO2FallingBank: GPIOBank = [None, ..32];

enum GPIOInterruptHandlerBank {
  GPIO0Rising,
  GPIO0Falling,
  GPIO2Rising,
  GPIO2Falling,
}

impl GPIOInterruptHandlerBank {
  pub fn bank(self) -> &mut GPIOBank {
    unsafe {
      match self {
        GPIO0Rising  => &mut GPIO0RisingBank,
        GPIO0Falling => &mut GPIO0FallingBank,
        GPIO2Rising  => &mut GPIO2RisingBank,
        GPIO2Falling => &mut GPIO2FallingBank,
      }
    }
  }
}

fn set_gpio_handler(bank: GPIOInterruptHandlerBank, pin: u8,
    h: Option<&GPIOISRHandler>) {
  bank.bank()[pin as uint] = unsafe { transmute(h) };
}

// TODO(farcaller): h shouldn't be called from isr, it actually should be passed
// onto corresponding user task stack. DO NOT allow user code to run in MSP!
// TODO(farcaller): this code is made with cmd+c / cmd+v.
#[cfg(gpio_isr)]
#[inline(always)]
pub unsafe fn isr_gpio() {
  let mut rise0 = reg::GPIOINT.IO0IntStatR();
  let mut fall0 = reg::GPIOINT.IO0IntStatF();
  let mut rise2 = reg::GPIOINT.IO2IntStatR();
  let mut fall2 = reg::GPIOINT.IO2IntStatF();

  while rise0 > 0 {
    let bitloc = 31 - count_leading_zeroes(rise0);
    let bank = GPIO0Rising.bank();
    let handler = bank[bitloc];
    match handler {
      Some(h) => h(),
      None => (),
    }
    let bit: u32 = 1 << bitloc;
    reg::GPIOINT.set_IO0IntClr(bit);
    rise0 -= bit;
  }

  while fall0 > 0 {
    let bitloc = 31 - count_leading_zeroes(fall0);
    let bank = GPIO0Falling.bank();
    let handler = bank[bitloc];
    match handler {
      Some(h) => h(),
      None => (),
    }
    let bit: u32 = 1 << bitloc;
    reg::GPIOINT.set_IO0IntClr(bit);
    fall0 -= bit;
  }

  while rise2 > 0 {
    let bitloc = 31 - count_leading_zeroes(rise2);
    let bank = GPIO2Rising.bank();
    let handler = bank[bitloc];
    match handler {
      Some(h) => h(),
      None => (),
    }
    let bit: u32 = 1 << bitloc;
    reg::GPIOINT.set_IO2IntClr(bit);
    rise2 -= bit;
  }

  while fall2 > 0 {
    let bitloc = 31 - count_leading_zeroes(fall2);
    let bank = GPIO2Falling.bank();
    let handler = bank[bitloc];
    match handler {
      Some(h) => h(),
      None => (),
    }
    let bit: u32 = 1 << bitloc;
    reg::GPIOINT.set_IO2IntClr(bit);
    fall2 -= bit;
  }
}

#[inline(always)]
fn count_leading_zeroes(val: u32) -> u8 {
  let out: u32;
  unsafe {
    asm!("clz $0, $1" : "=r"(out) : "r"(val) :: "volatile");
  }
  out as u8
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(GPIO: FIODIR, _pad_0, _pad_1, _pad_2, FIOMASK, FIOPIN, FIOSET, FIOCLR)
  reg_rw!(GPIO, FIODIR,  set_FIODIR,  FIODIR)
  reg_rw!(GPIO, FIOMASK, set_FIOMASK, FIOMASK)
  reg_rw!(GPIO, FIOPIN,  set_FIOPIN,  FIOPIN)
  reg_rw!(GPIO, FIOSET,  set_FIOSET,  FIOSET)
  reg_rw!(GPIO, FIOCLR,  set_FIOCLR,  FIOCLR)

  ioreg!(GPIOINTReg: IOIntStatus,
      IO0IntStatR, IO0IntStatF, IO0IntClr, IO0IntEnR, IO0IntEnF,
      _pad_0, _pad_1, _pad_2,
      IO2IntStatR, IO2IntStatF, IO2IntClr, IO2IntEnR, IO2IntEnF)
  reg_r!( GPIOINTReg, IOIntStatus,                IOIntStatus)

  reg_r!( GPIOINTReg, IO0IntStatR,                IO0IntStatR)
  reg_r!( GPIOINTReg, IO0IntStatF,                IO0IntStatF)
  reg_w!( GPIOINTReg,              set_IO0IntClr, IO0IntClr)
  reg_rw!(GPIOINTReg, IO0IntEnR,   set_IO0IntEnR, IO0IntEnR)
  reg_rw!(GPIOINTReg, IO0IntEnF,   set_IO0IntEnF, IO0IntEnF)

  reg_r!( GPIOINTReg, IO2IntStatR,                IO2IntStatR)
  reg_r!( GPIOINTReg, IO2IntStatF,                IO2IntStatF)
  reg_w!( GPIOINTReg,              set_IO2IntClr, IO2IntClr)
  reg_rw!(GPIOINTReg, IO2IntEnR,   set_IO2IntEnR, IO2IntEnR)
  reg_rw!(GPIOINTReg, IO2IntEnF,   set_IO2IntEnF, IO2IntEnF)

  extern {
    #[link_name="iomem_GPIO0"] pub static GPIO0: GPIO;
    #[link_name="iomem_GPIO1"] pub static GPIO1: GPIO;
    #[link_name="iomem_GPIO2"] pub static GPIO2: GPIO;
    #[link_name="iomem_GPIO3"] pub static GPIO3: GPIO;
    #[link_name="iomem_GPIO4"] pub static GPIO4: GPIO;

    #[link_name="iomem_GPIOINT"] pub static GPIOINT: GPIOINTReg;
  }
}
