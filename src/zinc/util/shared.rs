// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

//! Concurrency-friendly shared state

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::marker::{Sync, Send};

use hal::cortex_m3::irq::NoInterrupts;

/// This allows safe sharing of state, ensuring access occurs only
/// when in a critical section.
#[allow(missing_docs)]
pub struct Shared<T> {
  pub value: UnsafeCell<T>,
}

/// A reference to a shared value
pub struct SharedRef<'a, T: 'a> {
  ptr: &'a Shared<T>,
  #[allow(dead_code)]
  crit: &'a NoInterrupts
}

impl<T> Shared<T> {
  /// Create a new `Shared` value
  pub fn new(value: T) -> Shared<T> {
    Shared {
      value: UnsafeCell::new(value),
    }
  }

  /// Borrow a reference to the value
  pub fn borrow<'a>(&'a self, crit: &'a NoInterrupts) -> SharedRef<'a, T> {
    SharedRef {ptr: self, crit: crit}
  }
}

impl<'a, T> Deref for SharedRef<'a, T> {
  type Target = T;
  fn deref<'b>(&'b self) -> &'b T {
    unsafe {
      &*self.ptr.value.get()
    }
  }
}

impl<'a, T> DerefMut for SharedRef<'a, T> {
  fn deref_mut<'b>(&'b mut self) -> &'b mut T {
    unsafe {
      &mut *self.ptr.value.get()
    }
  }
}

unsafe impl<T: Send> Sync for Shared<T> {}
