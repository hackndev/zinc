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

//! A cell that with volatile setter and getter.

use core::intrinsics::{volatile_load, volatile_store};

/// This structure is used to represent a hardware register.
/// It is mostly used by the ioreg family of macros.
#[derive(Clone, Copy)]
pub struct VolatileCell<T> {
  value: T,
}

impl<T> VolatileCell<T> {
  /// Create a cell with initial value.
  pub fn new(value: T) -> VolatileCell<T> {
    VolatileCell {
      value: value,
    }
  }

  /// Get register value.
  #[inline]
  pub fn get(&self) -> T {
    unsafe {
      volatile_load(&self.value)
    }
  }

  /// Set register value.
  #[inline]
  pub fn set(&self, value: T) {
    unsafe {
      volatile_store(&self.value as *const T as *mut T, value)
    }
  }
}
