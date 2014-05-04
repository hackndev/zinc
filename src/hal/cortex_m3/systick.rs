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

pub static CALIBRATED: u32 = 0xffffffff;

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

pub fn enable() {
  reg::SYSTICK.set_CONTROL(reg::SYSTICK.CONTROL() | 1);
}

pub fn enable_irq() {
  reg::SYSTICK.set_CONTROL(reg::SYSTICK.CONTROL() | 0b010);
}

pub fn disable_irq() {
  reg::SYSTICK.set_CONTROL(reg::SYSTICK.CONTROL() & !0b010);
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(SYSTICKReg: CONTROL, RELOAD, CURRENT, CALIBRATION)
  reg_rw!(SYSTICKReg, CONTROL,     set_CONTROL, CONTROL)
  reg_rw!(SYSTICKReg, RELOAD,      set_RELOAD,  RELOAD)
  reg_rw!(SYSTICKReg, CURRENT,     set_CURRENT, CURRENT)
  reg_r!( SYSTICKReg, CALIBRATION,              CALIBRATION)

  extern {
    #[link_name="armmem_SYSTICK"] pub static SYSTICK: SYSTICKReg;
  }
}
