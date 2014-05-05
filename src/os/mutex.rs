#![feature(macro_rules)]
 
use hal::cortex_m3::sched::CritSection;
use os::task::{TaskDescriptor, Tasks};
use lib::queue::{Queue, Node};
use std::option::{Option, None, Some};
use std::cell::Cell;
use std::ops::Drop;

pub struct Mutex {
  owner: Cell<Option<*mut TaskDescriptor>>,
  waiting: Queue<*mut TaskDescriptor>
}

#[must_use]
pub struct Guard<'a>(&'a Mutex);

impl Mutex {
  /*
   * This is a bit subtle: We need to add ourselves to the mutex's
   * waiting list. To do this we allocate a list item on the local
   * stack, append it to the waiting list, and block. When the task
   * before us unlocks the mutex, they will wake us up. Finally, when
   * we are executing again we remove our entry from the list.
   */
  pub fn lock<'a>(&'a self) -> Guard<'a> {
    unsafe {
      let crit = match self.owner.get() {
        None    => CritSection::new(),
        Some(_) => {
          let crit = CritSection::new();
          let mut waiting = Node::new(Tasks.current_task() as *mut TaskDescriptor);
          self.waiting.push(&mut waiting, &crit);
          Tasks.current_task().block(crit);

          /*
           * Note that there is a small window here between being
           * awoken by the unlocking thread and reentering a critical
           * section. An interrupt could fire within this window but
           * since the unlocking thread retains ownership of the mutex
           * there is no risk of a third-party sneaking in.
           */
          let crit = CritSection::new();
          self.waiting.pop(&crit);
          crit
        }
      };

      self.owner.set(Some(Tasks.current_task() as *mut TaskDescriptor));
      Guard(self)
    }
  }

  pub fn try_lock<'a>(&'a self) -> Option<Guard<'a>> {
    unsafe {
      match self.owner.get() {
        None    => {
          let crit = CritSection::new();
          self.owner.set(Some(Tasks.current_task() as *mut TaskDescriptor));
          Some(Guard(self))
        }
        _       => None
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
      let crit = CritSection::new();
      match self.waiting.peek() {
        None => self.owner.set(None),
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
    let &Guard(ref mutex) = self;
    mutex.unlock();
  }
}
