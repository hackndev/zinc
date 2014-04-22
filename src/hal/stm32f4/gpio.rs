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

use core::fail;
use hal;
use hal::stm32f4::clocking;

#[path="../../lib/ioreg.rs"]
mod ioreg;

mod reg {
  use core::{volatile_load, volatile_store};

  ioreg!(GPIO: MODER, OTYPER, OSPEEDER, PUPDR, IDR, ODR, BSRR, LCKR, AFRL, AFRH)
  reg_rw!(GPIO, MODER,    set_MODER,    MODER)
  reg_rw!(GPIO, OTYPER,   set_OTYPER,   OTYPER)
  reg_rw!(GPIO, OSPEEDER, set_OSPEEDER, OSPEEDER)
  reg_rw!(GPIO, PUPDR,    set_PUPDR,    PUPDR)
  reg_rw!(GPIO, IDR,      set_IDR,      IDR)
  reg_rw!(GPIO, ODR,      set_ODR,      ODR)
  reg_rw!(GPIO, BSRR,     set_BSRR,     BSRR)
  reg_rw!(GPIO, LCKR,     set_LCKR,     LCKR)
  reg_rw!(GPIO, AFRL,     set_AFRL,     AFRL)
  reg_rw!(GPIO, AFRH,     set_AFRH,     AFRH)

  define_reg!(GPIO_A: GPIO @ 0x40020000)
  define_reg!(GPIO_B: GPIO @ 0x40020400)
  define_reg!(GPIO_C: GPIO @ 0x40020800)
  define_reg!(GPIO_D: GPIO @ 0x40020c00)
  define_reg!(GPIO_E: GPIO @ 0x40021000)
  define_reg!(GPIO_F: GPIO @ 0x40021400)
  define_reg!(GPIO_G: GPIO @ 0x40021800)
  define_reg!(GPIO_H: GPIO @ 0x40021c00)
  define_reg!(GPIO_I: GPIO @ 0x40022000)
  // define_reg!(GPIO_J: GPIO @ 0x40022400)
  // define_reg!(GPIO_K: GPIO @ 0x40022800)
}

pub enum Port {
  PortA,
  PortB,
  PortC,
  PortD,
  PortE,
  PortF,
  PortG,
  PortH,
  PortI,
}

pub enum Function {
  GPIOIn,
  GPIOOut,
  // TODO(farcaller): alt functions
}

fn port_to_reg(port: Port) -> *mut reg::GPIO {
  match port {
    PortA => reg::GPIO_A,
    PortB => reg::GPIO_B,
    PortC => reg::GPIO_C,
    PortD => reg::GPIO_D,
    PortE => reg::GPIO_E,
    PortF => reg::GPIO_F,
    PortG => reg::GPIO_G,
    PortH => reg::GPIO_H,
    PortI => reg::GPIO_I,
  }
}

pub fn configure(pin: &hal::gpio::Pin) {
  if pin.pin > 15 {
    fail::abort();
  }

  let bit: u32 = match pin.function {
    GPIOOut => 0b01 << (pin.pin*2),
    GPIOIn  => 0b00 << (pin.pin*2),
  };
  let mask: u32 = !(0b11 << (pin.pin*2));
  let reg = port_to_reg(pin.port);
  let val: u32 = unsafe { (*reg).MODER() };
  let new_val: u32 = val & mask | bit;
  unsafe { (*reg).set_MODER(new_val) };
}

pub fn set_high(cpin: &hal::gpio::ConnectedPin) {
  let bit: u32 = 1 << cpin.pin();
  let reg = port_to_reg(cpin.port());
  unsafe { (*reg).set_BSRR(bit) };
}

pub fn set_low(cpin: &hal::gpio::ConnectedPin) {
  let bit: u32 = 1 << (cpin.pin() + 16);
  let reg = port_to_reg(cpin.port());
  unsafe { (*reg).set_BSRR(bit) };
}

pub fn enable_port_clock(port: Port) {
  let peripheral = match port {
    PortA => clocking::GPIO_A,
    PortB => clocking::GPIO_B,
    PortC => clocking::GPIO_C,
    PortD => clocking::GPIO_D,
    PortE => clocking::GPIO_E,
    PortF => clocking::GPIO_F,
    PortG => clocking::GPIO_G,
    PortH => clocking::GPIO_H,
    PortI => clocking::GPIO_I,
  };
  clocking::set_peripheral_clock(peripheral, true);
}
