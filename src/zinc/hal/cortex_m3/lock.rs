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
use core::kinds::marker::InvariantType;

/// A lock
pub struct Lock {
    locked: Unsafe<u32>
}

#[must_use]
pub struct Guard<'a>(&'a Lock);

pub static STATIC_LOCK: Lock = Lock { locked: Unsafe { value: 0, marker1: InvariantType } };

#[cfg(not(test))]
#[inline(always)]
unsafe fn exclusive_load(addr: *const u32) -> u32 {
  let mut value: u32;
  asm!("ldrex $0, [$1]"
       : "=r"(value)
       : "r"(addr)
       :
       : "volatile"
       );
  value
}

#[cfg(test)]
unsafe fn exclusive_load(addr: *const u32) -> u32 { unimplemented!() }

#[cfg(not(test))]
#[inline(always)]
unsafe fn exclusive_store(addr: *mut u32, value: u32) -> bool {
  let mut success: u32;
  asm!("strex $0, $2, [$1]"
       : "=r"(success)
       : "r"(addr), "r"(value)
       :
       : "volatile"
       );
  success == 0
}

#[cfg(test)]
unsafe fn exclusive_store(addr: *mut u32, value: u32) -> bool {
  unimplemented!()
}

impl Lock {
  pub fn new() -> Lock {
    Lock { locked: Unsafe::new(0) }
  }

  pub fn try_lock<'a>(&'a self) -> Option<Guard<'a>> {
    unsafe {
      let ptr: *mut u32 = self.locked.get();
      let locked = exclusive_load(&*ptr) == 1;
      let success = exclusive_store(ptr, 1);
      if !locked && success {
        return Some(Guard(self));
      } else {
        return None;
      }
    }
  }

  fn unlock<'a>(&'a self) {
    unsafe {
      loop {
        let ptr: *mut u32 = self.locked.get();
        let _locked = exclusive_load(&*ptr) == 1;
        let success = exclusive_store(ptr, 0);
        if success {
          break;
        }
      }
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
