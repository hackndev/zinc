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

//! Cortex-M3 specific support code for scheduler.

use core::ops::Drop;
use core::intrinsics::abort;

use os::task::Task;
use super::scb;

/// Force context switch. Triggers PendSV interrupt.
#[inline(always)]
pub fn switch_context() {
   scb::set_pendsv(true);
}

/// Sets task stack pointer (PSP).
#[cfg(not(test))]
#[inline(always)]
pub fn set_task_stack_pointer(val: u32) {
  unsafe { asm!("msr psp, $0" :: "r"(val) :: "volatile") };
}

#[cfg(test)]
pub fn set_task_stack_pointer(val: u32) { unimplemented!() }

/// Returns task stack pointer (PSP).
#[cfg(not(test))]
#[inline(always)]
pub fn get_task_stack_pointer() -> u32 {
  let mut val: u32;
  unsafe { asm!("mrs $0, psp" : "=r"(val) ::: "volatile") };
  val
}

#[cfg(test)]
pub fn get_task_stack_pointer() -> u32 { unimplemented!() }

/// Returns current stack pointer (SP, which may be PSP or MSP).
#[cfg(not(test))]
#[inline(always)]
pub fn get_current_stack_pointer() -> u32 {
  let mut val: u32;
  unsafe { asm!("mov $0, sp" : "=r"(val) ::: "volatile") };
  val
}

#[cfg(test)]
pub fn get_current_stack_pointer() -> u32 { unimplemented!() }

/// State, that's saved by hardware upon entering an ISR.
pub struct SavedState {
  pub r0: u32,
  pub r1: u32,
  pub r2: u32,
  pub r3: u32,
  pub r12: u32,
  pub lr: u32,
  pub pc: u32,
  pub psr: u32,
}

impl SavedState {
  #[inline(always)]
  pub fn new(t: Task, arg: u32) -> SavedState {
    SavedState {
      r0:  arg,
      r1:  0,
      r2:  0,
      r3:  0,
      r12: 0,
      lr:  task_finished as u32,
      pc:  t as u32,
      psr: 0x01000000,  // thumb state
    }
  }
}

// TODO(farcaller): this should actually kill the task.
// TODO(bgamari): It should also unlock anything the task holds
/// Default handler for task that tries to return.
#[cfg(not(test))]
unsafe fn task_finished() {
  asm!("bkpt" :::: "volatile");
}

#[cfg(test)]
unsafe fn task_finished() { unimplemented!() }

