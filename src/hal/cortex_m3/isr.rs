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

use core::option::*;

extern {
  fn __STACK_BASE();
  fn main();
  fn _boot_checksum();
}

static ISRCount: uint = 16;

#[link_section=".isr_vector"]
#[no_mangle]
pub static ISRVectors: [Option<extern unsafe fn()>, ..ISRCount] = [
  Some(__STACK_BASE),
  Some(main),             // Reset
  Some(isr_nmi),          // NMI
  Some(isr_hardfault),    // Hard Fault
  None,                   // CM3 Memory Management Fault
  None,                   // CM3 Bus Fault
  None,                   // CM3 Usage Fault
  Some(_boot_checksum),   // NXP Checksum code
  None,                   // Reserved
  None,                   // Reserved
  None,                   // Reserved
  Some(isr_hang),         // SVCall
  None,                   // Reserved for debug
  None,                   // Reserved
  Some(isr_hang),         // PendSV
  Some(isr_hang),         // SysTick
];

#[no_mangle]
#[no_split_stack]
pub extern "C" fn isr_nmi() {
  loop {
    unsafe { asm!("bkpt") }
  }
}

#[no_mangle]
#[no_split_stack]
pub extern "C" fn isr_hardfault() {
  loop {
    unsafe { asm!("bkpt") }
  }
}

#[no_mangle]
#[no_split_stack]
pub extern "C" fn isr_hang() {
  loop {
    unsafe { asm!("bkpt") }
  }
}
