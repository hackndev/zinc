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

//! Rust support code for scheduler. This lives here instead of sched.s to allow
//  inlining.

use os::task::Task;

/// Force context switch. This triggers PendSV.
#[inline(always)]
pub fn switch_context() {
  let icsr_addr: u32 = 0xE000ED04;
  let icsr_reg: *mut u32 = icsr_addr as *mut u32;

  unsafe { *icsr_reg = 1 << 28 };
}

/// Sets task stack pointer (PSP).
#[inline(always)]
pub fn set_task_stack_pointer(val: u32) {
  unsafe { asm!("msr psp, $0" :: "r"(val)) };
}

/// Returns task stack pointer (PSP).
#[inline(always)]
pub fn get_task_stack_pointer() -> u32 {
  let mut val: u32;
  unsafe { asm!("mrs $0, psp" : "=r"(val)) };
  val
}

/// Returns current stack pointer (SP, which may be PSP or MSP).
#[inline(always)]
pub fn get_current_stack_pointer() -> u32 {
  let mut val: u32;
  unsafe { asm!("mov $0, sp" : "=r"(val)) };
  val
}

/// State, that's saved by hardware.
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

unsafe fn task_finished() {
  asm!("bkpt" :::: "volatile");
}
