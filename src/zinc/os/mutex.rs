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

//! Mutexes

pub use os::mutex::internal::{MUTEX_INIT, Mutex, Guard};

#[cfg(multitasking)]
mod internal {
  use core::marker;
  use core::ty::Unsafe;
  use core::kinds::Sync;
  use core::option::Option::{None, Some};
  use core::ops::Drop;

  use hal::cortex_m3::irq::NoInterrupts;
  use os::task::{TaskDescriptor, Tasks};
  use util::queue::{Queue, Node};

  /// A mutex
  pub struct Mutex {
    owner: UnsafeCell<Option<*mut TaskDescriptor>>,
    waiting: Queue<*mut TaskDescriptor>
  }

  /// Static initializer
  pub const MUTEX_INIT: Mutex = Mutex {
    owner: UnsafeCell {value: None},
    waiting: Queue {
      head: UnsafeCell {value: 0 as *mut Node<*mut TaskDescriptor>,},
      tail: UnsafeCell {value: 0 as *mut Node<*mut TaskDescriptor>,},
    }
  };

  /// A mutex lock
  #[must_use]
  pub struct Guard<'a> {
    mutex: &'a Mutex
  }

  impl Mutex {
    /// Create a new mutex
    pub fn new() -> Mutex {
      Mutex {
        owner: UnsafeCell::new(None),
        waiting: Queue::new(),
      }
    }

    /// Take a mutex lock
    pub fn lock<'a>(&'a self) -> Guard<'a> {
      /*
       * This is a bit subtle: We need to add ourselves to the mutex's
       * waiting list. To do this we allocate a list item on the local
       * stack, append it to the waiting list, and block. When the task
       * before us unlocks the mutex, they will wake us up. Finally, when
       * we are executing again we remove our entry from the list.
       */
      unsafe {
        // we need the critical section until the end of this function
        let _crit = match *self.owner.get() {
          None    => NoInterrupts::new(),
          Some(_) => {
            let crit = NoInterrupts::new();
            let mut waiting = Node::new(Tasks.current_task() as *mut TaskDescriptor);
            self.waiting.push(&mut waiting, &crit);
            Tasks.current_task().block(crit); // drops crit

            /*
             * Note that there is a small window here between being
             * awoken by the unlocking thread and reentering a critical
             * section. An interrupt could fire within this window but
             * since the unlocking thread retains ownership of the mutex
             * there is no risk of a third-party sneaking in.
             */
            let crit = NoInterrupts::new();
            self.waiting.pop(&crit);
            crit
          }
        };

        *self.owner.get() = Some(Tasks.current_task() as *mut TaskDescriptor);
        Guard { mutex: self }
      }
    }

    /// Try to take a mutex lock, returning `None` is unsuccessful
    pub fn try_lock<'a>(&'a self) -> Option<Guard<'a>> {
      unsafe {
        match *self.owner.get() {
          None => {
            let _crit = NoInterrupts::new();
            *self.owner.get() = Some(Tasks.current_task() as *mut TaskDescriptor);
            Some(Guard { mutex: self })
          }
          _ => None
        }
      }
    }

    /*
     * Here we release ownership of the mutex only if there is no one
     * waiting on it. Otherwise we retain to ensure there is no race
     * between waking up the waiting thread and it claiming ownership.
     */
    fn unlock(&self) {
      unsafe {
        let crit = NoInterrupts::new();
        match self.waiting.peek() {
          None => *self.owner.get() = None,
          Some(nextTask) => {
            let mut task = *(*nextTask).data;
            task.unblock(&crit);
          }
        }
      }
    }
  }

  #[unsafe_destructor]
  impl<'a> Drop for Guard<'a> {
    #[inline]
    fn drop(&mut self) {
      self.mutex.unlock();
    }
  }

  impl Sync for Mutex { }
}

#[cfg(not(multitasking))]
mod internal {
  use core::marker::Sync;
  use core::option::Option::{self, None, Some};
  use core::ops::Drop;
  use core::intrinsics::abort;
  use core::cell::UnsafeCell;

  /// A mutex
  pub struct Mutex {
    taken: UnsafeCell<bool>,
  }

  pub const MUTEX_INIT: Mutex = Mutex { taken: UnsafeCell { value: false  } };

  /// A mutex lock
  #[must_use]
  pub struct Guard<'a> {
    mutex: &'a Mutex,
  }

  impl Mutex {
    /// Create a new mutex
    pub fn new() -> Mutex {
      Mutex { taken: UnsafeCell::new(false) }
    }

    /// Take a mutex lock
    pub fn lock<'a>(&'a self) -> Guard<'a> {
      unsafe {
        if *self.taken.get() {
          abort();
        }
        *self.taken.get() = true;
      }
      Guard {
        mutex: self
      }
    }

    /// Try to take a mutex lock, returning `None` is unsuccessful
    pub fn try_lock<'a>(&'a self) -> Option<Guard<'a>> {
      unsafe {
        if *self.taken.get() {
          None
        } else {
          Some(self.lock())
        }
      }
    }
  }

  #[unsafe_destructor]
  impl<'a> Drop for Guard<'a> {
    #[inline]
    fn drop(&mut self) {
      unsafe {
        *self.mutex.taken.get() = false;
      }
    }
  }

  unsafe impl Sync for Mutex { }
}
