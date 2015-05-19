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

//! Stack layout information.

use core::intrinsics::transmute;

extern {
  fn __STACK_BASE();
  static mut __STACK_LIMIT: u32;
}

/// Returns the address of main stack base (end of ram).
pub fn stack_base() -> usize {
    unsafe {
        transmute(__STACK_BASE)
    }
}

/// Returns the current stack limit.
pub fn stack_limit() -> u32 {
  unsafe { __STACK_LIMIT }
}

/// Sets the current stack limit.
pub fn set_stack_limit(val: u32) {
  unsafe { __STACK_LIMIT = val }
}
