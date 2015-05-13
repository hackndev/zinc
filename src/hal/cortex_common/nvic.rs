// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Harris <mail@bharr.is>
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

//! Interface to Nested Vector Interrupt Controller.
//!
//! NVIC memory location is 0xE000_E000.
//  Link: http://infocenter.arm.com/help/topic/com.arm.doc.dui0552a/CIHIGCIF.html

#[inline(always)]
fn get_reg() -> &'static reg::NVIC {
  unsafe { &*(0xE000_E000 as *mut reg::NVIC) }
}

/// Enable an interrupt
pub fn enable_irq(irqn: usize) {
  get_reg().iser[irqn / 32].clear_iser(irqn % 32);
}

/// Disable an interrupt
pub fn disable_irq(irqn: usize) {
  get_reg().icer[irqn / 32].clear_icer(irqn % 32);
}

/// Return whether the given interrupt is enabled
pub fn is_enabled(irqn: usize) -> bool {
  get_reg().iser[irqn / 32].iser(irqn % 32)
}

/// Clear the pending flag for the given interrupt
pub fn clear_pending(irqn: usize) {
  get_reg().icpr[irqn / 32].clear_icpr(irqn % 32);
}

/// Return whether the given interrupt is pending
pub fn is_pending(irqn: usize) -> bool {
  get_reg().ispr[irqn / 32].ispr(irqn % 32)
}

/// Return whether the given interrupt is active
pub fn is_active(irqn: usize) -> bool {
  get_reg().iabr[irqn / 32].iabr(irqn % 32)
}

/// Set the priority for the given interrupt
pub fn set_priority(irqn: usize, prio: u8) {
  get_reg().ipr[irqn / 4].set_ipr(irqn % 4, prio as u32);
}

/// Return the priority for the given interrupt
pub fn get_priority(irqn: usize) -> u8 {
  get_reg().ipr[irqn / 4].ipr(irqn % 4) as u8
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(NVIC = {
    0x100     => reg32 iser[8] {      //! Interrupt set enable register
      0..31   => iser[32]: set_to_clear,
    }
    0x180     => reg32 icer[8] {      //! Interrupt clear enable register
      0..31   => icer[32]: set_to_clear,
    }
    0x200     => reg32 ispr[8] {      //! Interrupt set pending register
      0..31   => ispr[32]: set_to_clear,
    }
    0x280     => reg32 icpr[8] {      //! Interrupt clear pending register
      0..31   => icpr[32]: set_to_clear,
    }
    0x300     => reg32 iabr[8] {      //! Interrupt active bit register
      0..31   => iabr[32]: ro,
    }
    0x400     => reg32 ipr[8] {       //! Interrupt priority register
      0..31   => ipr[4],
    }
    0xF00     => reg32 stir[8] {      //! Software triggered interrupt register
      0..8    => stir,
    }
  });
}
