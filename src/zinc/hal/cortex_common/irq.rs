// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@smart-cactus.org>
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

//! Disabling and enabling interrupts

use core::ops::Drop;
#[cfg(not(test))]
use core::intrinsics::abort;

/// Phantom type to indicate that interrupts are disabled.
pub struct NoInterrupts {
  #[allow(dead_code)]
  contents: ()
}

impl NoInterrupts {
  /// Start a new critical section
  pub fn new() -> NoInterrupts {
    unsafe {
      disable_irqs();
    }
    NoInterrupts { contents: () }
  }
}

impl Drop for NoInterrupts {
  fn drop(&mut self) {
    unsafe {
      enable_irqs();
    }
  }
}

#[cfg(not(test))]
static mut irq_level : usize = 0;

/// Disables all interrupts except Reset, HardFault, and NMI.
/// Note that this is reference counted: if `disable_irqs` is called
/// twice then interrupts will only be re-enabled upon the second call
/// to `enable_irqs`.
#[cfg(not(test))]
#[inline(always)]
unsafe fn disable_irqs() {
  asm!("cpsid i" :::: "volatile");
  irq_level += 1;
}

#[cfg(test)]
unsafe fn disable_irqs() { unimplemented!() }

/// Enables all interrupts except Reset, HardFault, and NMI.
#[cfg(not(test))]
#[inline(always)]
unsafe fn enable_irqs() {
  if irq_level == 0 {
    abort();
  }
  // There is no race condition here as we know that interrupts are
  // disabled.
  irq_level -= 1;
  if irq_level == 0 {
    asm!("cpsie i" :::: "volatile");
  }
}

#[cfg(test)]
unsafe fn enable_irqs() { unimplemented!() }
