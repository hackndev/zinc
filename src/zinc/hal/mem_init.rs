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

//! Helper functions for memory initialisation.

use hal::stack::set_stack_limit;

extern {
  static _data_load: u32;
  static mut _data: u32;
  static mut _edata: u32;
  static mut _bss: u32;
  static mut _ebss: u32;

  static _eglobals: u32;
}

/// Helper function to initialise the stack limit.
#[inline(always)]
pub fn init_stack() {
  set_stack_limit((&_eglobals as *const u32) as u32);
}

/// Helper function to initialize memory.
/// Copies `.data` sections in to RAM and initializes `.bss` sections to zero.
#[inline(always)]
pub fn init_data() {
  unsafe {
    let mut load_addr: *const u32 = &_data_load;
    let mut mem_addr: *mut u32 = &mut _data;
    while mem_addr < &mut _edata as *mut u32 {
      *mem_addr = *load_addr;
      mem_addr = ((mem_addr as u32) + 4) as *mut u32;
      load_addr = ((load_addr as u32) + 4) as *const u32;
    }

    mem_addr = &mut _bss as *mut u32;
    while mem_addr < &mut _ebss as *mut u32 {
      *mem_addr = 0u32;
      mem_addr = ((mem_addr as u32) + 4) as *mut u32;
    }
  }
}
