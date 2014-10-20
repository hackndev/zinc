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

use core::option::{Option,None,Some};

/// Initialize systick timer.
///
/// After this call system timer will be disabled, and needs to be enabled manual. SysTick irq will
/// be disabled and needs to be enabled manually, too.
///
/// Arguments:
///
///  * reload: Reload value for the timer
pub fn setup(reload: u32) {
  reg::SYSTICK.csr.set_enable(false).set_tickint(false).set_clksource(reg::CPU);

  reg::SYSTICK.rvr.set_reload(reload);
  reg::SYSTICK.cvr.set_current(0);
}

/// Read ten millisecond calibration value from hardware
pub fn ten_ms() -> Option<u32> {
  let calib = reg::SYSTICK.calib.tenms();
  match calib {
    0 => None,
    val => Some(val)
  }
}

/// Enables the timer.
pub fn enable() {
  reg::SYSTICK.csr.set_enable(true);
}

/// Disable the timer.
pub fn disable() {
  reg::SYSTICK.csr.set_enable(false);
}

/// Enables interrupts generation for timer.
pub fn enable_irq() {
  reg::SYSTICK.csr.set_tickint(true);
}

/// Disables interrupts generation for timer, which is still ticking.
pub fn disable_irq() {
  reg::SYSTICK.csr.set_tickint(false);
}

/// Gets the current 24bit systick value.
pub fn get_current() -> u32 {
  reg::SYSTICK.cvr.current()
}

/// Checks if the timer has been triggered since last call.
/// The flag is cleared when this is called.
pub fn tick() -> bool {
  reg::SYSTICK.csr.countflag()
}

#[allow(dead_code)]
mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(SYSTICK = {
    /// SysTick Control and Status Register
    0x0 => reg32 csr
    {
      16 => countflag : ro,   //= Returns 1 if timer counted to 0
                              //= since last time this was read.
      2  => clksource : rw {
        0 => External,        //= External clock
        1 => CPU,             //= CPU clock
      },
      1 => tickint : rw,      //= Enable SysTick exception
      0 => enable : rw
    },

    /// Reload Value Register
    0x4 => reg32 rvr {
      23..0 => reload : rw    //= Reload value
    }

    /// Current Value Register
    0x8 => reg32 cvr {
      31..0 => current : rw   //= Current timer value
    },

    0xc => reg32 calib {
      31    => noref : ro,    //= If 1, the reference clock is not provided
      30    => skew : ro,     //= If 1, the calibration value is inexact
      23..0 => tenms : ro,    //= An optional Reload value for 10ms (100Hz) timing
                              //= If zero calibration value not known
    },
  })

  extern {
    #[link_name="armmem_SYSTICK"] pub static SYSTICK: SYSTICK;
  }
}
