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

//! Interface to SysTick timer.

#[path="../../lib/ioreg.rs"] mod ioreg;

pub static CALIBRATED: u32 = 0xffffffff;

pub fn setup(calibration: u32, enable_irq: bool) {
  reg::SysTick.set_control(0b100);  // disabled, no interrupt
  let reload_val: u32 = match calibration {
    CALIBRATED => reg::SysTick.calibration() & 0xffffff,
    val => val,
  };
  reg::SysTick.set_reload(reload_val);
  reg::SysTick.set_current(0);
  if enable_irq {
    reg::SysTick.set_control(0b110);
  }
}

pub fn enable() {
  reg::SysTick.set_control(reg::SysTick.control() | 1);
}

pub fn enable_irq() {
  reg::SysTick.set_control(reg::SysTick.control() | 0b010);
}

pub fn disable_irq() {
  reg::SysTick.set_control(reg::SysTick.control() & !0b010);
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(SysTickReg: control, reload, current, calibration)
  reg_rw!(SysTickReg, control,     set_control, control)
  reg_rw!(SysTickReg, reload,      set_reload,  reload)
  reg_rw!(SysTickReg, current,     set_current, current)
  reg_r!( SysTickReg, calibration,              calibration)

  extern {
    #[link_name="iomem_SYSTICK"] pub static SysTick: SysTickReg;
  }
}
