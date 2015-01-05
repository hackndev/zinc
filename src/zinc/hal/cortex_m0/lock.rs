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

use core::ty::Unsafe;
use core::option::Option::{self, Some, None};
use core::ops::Drop;
use core::kinds::Share;

/// A lock. Note that this disables interrupts. Consequently, a task
/// dying (e.g. by running out of stack space) while holding a lock
/// may cause a deadlock.
pub struct Lock {
  locked: Unsafe<bool>
}

#[must_use]
pub struct Guard<'a>(&'a Lock);

pub static STATIC_LOCK: Lock = Lock { locked: Unsafe { value: false, marker1: InvariantType } };

impl Lock {
  pub fn new() -> Lock {
    Lock { locked: Unsafe::new(false) }
  }

  pub fn try_lock<'a>(&'a self) -> Option<Guard<'a>> {
    unsafe {
      let crit = NoInterrupts::new();
      let locked = self.locked.get();
      match *locked {
        true  => return None,
        false => {
          *locked = true;
          return Some(Guard(self));
        }
      }
    }
  }

  fn unlock<'a>(&'a self) {
    unsafe {
      let crit = NoInterrupts::new();
      *self.locked.get() = false;
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for Guard<'a> {
  fn drop(&mut self) {
    let &Guard(ref lock) = self;
    lock.unlock();
  }
}

impl Share for Lock { }
