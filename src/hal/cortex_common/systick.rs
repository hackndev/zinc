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

//! Interface to SYSTICK timer.

#[path="../../lib/ioreg.rs"] mod ioreg;

/// A constant that requests to use hardware calibration value.
/// Note that not all core implementations support this.
pub static CALIBRATED: u32 = 0xffffffff;

/// Initializes systick timer.
///
/// Arguments:
///
///  * calibration: Timer reload value, or `CALIBRATED` to use the device 10ms
///                 calibrated value.
///  * enable_irq: true, if IRQ should be initially enabled.
pub fn setup(calibration: u32, enable_irq: bool) {
  reg::SYSTICK.set_CONTROL(0b100);  // disabled, no interrupt
  let reload_val: u32 = match calibration {
    CALIBRATED => reg::SYSTICK.CALIBRATION() & 0xffffff,
    val => val,
  };
  reg::SYSTICK.set_RELOAD(reload_val);
  reg::SYSTICK.set_CURRENT(0);
  if enable_irq {
    reg::SYSTICK.set_CONTROL(0b110);
  }
}

/// Enables the timer.
pub fn enable() {
  reg::SYSTICK.set_CONTROL(reg::SYSTICK.CONTROL() | 1);
}

/// Enables interrupts generation for timer.
pub fn enable_irq() {
  reg::SYSTICK.set_CONTROL(reg::SYSTICK.CONTROL() | 0b010);
}

/// Disables interrupts generation for timer, which is still ticking.
pub fn disable_irq() {
  reg::SYSTICK.set_CONTROL(reg::SYSTICK.CONTROL() & !0b010);
}

/// Gets the current 24bit systick value.
pub fn get_current() -> u32 {
  reg::SYSTICK.CURRENT() & 0xFFFFFF
}

/// Checks if the timer has been triggered since last call.
/// The flag is cleared when this is called.
pub fn tick() -> bool {
  ((reg::SYSTICK.CONTROL() >> 16) & 0x1) == 1
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(SYSTICKReg: u32, CONTROL, RELOAD, CURRENT, CALIBRATION)
  reg_rw!(SYSTICKReg, u32, CONTROL,     set_CONTROL, CONTROL)
  reg_rw!(SYSTICKReg, u32, RELOAD,      set_RELOAD,  RELOAD)
  reg_rw!(SYSTICKReg, u32, CURRENT,     set_CURRENT, CURRENT)
  reg_r!( SYSTICKReg, u32, CALIBRATION,              CALIBRATION)

  extern {
    #[link_name="armmem_SYSTICK"] pub static SYSTICK: SYSTICKReg;
  }
}
