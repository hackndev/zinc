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

use std::option::{Option, Some, None};

extern {
  fn __STACK_BASE();
  fn main();
  fn _boot_checksum();

  fn isr_nmi();
  fn isr_hardfault();
  fn isr_mmfault();
  fn isr_busfault();
  fn isr_usagefault();
  fn isr_svcall();
  fn isr_pendsv();
  fn isr_systick();
}

#[no_mangle]
#[no_split_stack]
pub extern fn _default_fault() {
    unsafe {
        asm!("mrs r0, psp
             mrs r1, msp
             ldr r2, [r0, 0x18]
             ldr r3, [r1, 0x18]
             bkpt")
    }
}

#[no_mangle]
#[no_split_stack]
pub extern fn _default_system() {
    unsafe { asm!("bkpt") }
}

static ISRCount: uint = 16;

#[link_section=".isr_vector"]
#[no_mangle]
pub static ISRVectors: [Option<extern unsafe fn()>, ..ISRCount] = [
  Some(__STACK_BASE),
  Some(main),             // Reset
  Some(isr_nmi),          // NMI
  Some(isr_hardfault),    // Hard Fault
  Some(isr_mmfault),      // CM3 Memory Management Fault
  Some(isr_busfault),     // CM3 Bus Fault
  Some(isr_usagefault),   // CM3 Usage Fault
  Some(_boot_checksum),   // NXP Checksum code
  None,                   // Reserved
  None,                   // Reserved
  None,                   // Reserved
  Some(isr_svcall),       // SVCall
  None,                   // Reserved for debug
  None,                   // Reserved
  Some(isr_pendsv),       // PendSV
  Some(isr_systick),      // SysTick
];
