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

//! Condition variables

pub use os::cond_var::internal::{CondVar, COND_VAR_INIT};

#[cfg(multitasking)]
mod internal {
  use core::option::{None, Some};
  use core::ty::Unsafe;
  use core::marker;
  use core::marker::Sync;

  use hal::cortex_m3::sched::NoInterrupts;
  use util::queue::{Queue, Node};
  use os::task::{TaskDescriptor, Tasks};

  /// A condition variable
  pub struct CondVar {
    waiting: Queue<*mut TaskDescriptor>
  }

  /// Static initializer
  pub const COND_VAR_INIT: CondVar = CondVar {
    waiting: Queue {
      head: Unsafe { value: 0 as *mut Node<*mut TaskDescriptor>, marker1: marker::InvariantType },
      tail: Unsafe { value: 0 as *mut Node<*mut TaskDescriptor>, marker1: marker::InvariantType },
    }
  };

  impl CondVar {
    /// Create a new condition variable
    pub fn new() -> CondVar {
      CondVar { waiting: Queue::new() }
    }

    /// Wait on a condition variable.
    pub fn wait(&self) {
      /*
       * The signalling thread is responsible for removing the waiting
       * thread which ensures that a signal wakes up exactly one thread
       * whenever there is one waiting.
       */
      unsafe {
        let crit = NoInterrupts::new();
        let mut waiting = Node::new(Tasks.current_task() as *mut TaskDescriptor);
        self.waiting.push(&mut waiting, &crit);
        Tasks.current_task().block(crit);
      }
    }

    /// Wake up a thread waiting on a condition variable.
    pub fn signal(&self) {
      unsafe {
        let crit = NoInterrupts::new();
        match self.waiting.pop(&crit) {
          None => { },
          Some(task) => (*(*task).data).unblock(&crit)
        }
      }
    }

    /// Wake up all threads waiting on a condition variable.
    pub fn broadcast(&self) {
      unsafe {
        let crit = NoInterrupts::new();
        loop {
          match self.waiting.pop(&crit) {
            None => break,
            Some(task) => (*(*task).data).unblock(&crit)
          }
        }
      }
    }
  }

  impl Sync for CondVar {}
}

#[cfg(not(multitasking))]
mod internal {
  use core::marker;
  use core::marker::Sync;
  use core::cell::UnsafeCell;

  use util::support::wfi;

  /// A condition variable
  pub struct CondVar {
    waiting: UnsafeCell<bool>,
    nocopy: marker::NoCopy
  }

  /// Static initializer
  pub const COND_VAR_INIT: CondVar = CondVar {
    waiting: UnsafeCell { value: false },
    nocopy: marker::NoCopy,
  };

  impl CondVar {
    /// Create a new condition variable
    pub fn new() -> CondVar {
      CondVar {
        waiting: UnsafeCell::new(false),
        nocopy: marker::NoCopy,
      }
    }

    /// Wait on a condition variable.
    pub fn wait(&self) {
      unsafe {
        // TODO(bgamari): There is a race condition here
        *self.waiting.get() = true;
        while *self.waiting.get() {
          wfi();
        }
      }
    }

    /// Wake up a thread waiting on a condition variable.
    pub fn signal(&self) {
      unsafe {
        *self.waiting.get() = false;
      }
    }

    /// Wake up all threads waiting on a condition variable.
    pub fn broadcast(&self) {
      self.signal();
    }
  }

  unsafe impl Sync for CondVar {}
}
