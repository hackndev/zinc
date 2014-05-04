#![feature(globs, macro_rules)]
 
use hal::cortex_m3::sched::{disable_irqs, enable_irqs};
use os::task::{TaskDescriptor, Tasks, task_scheduler};
use lib::queue::*;
use std::option::{Option, None, Some};
use std::cell::{Cell};
use std::ops::{Drop};

pub struct StaticMutex {
  owner: Cell<Option<*mut TaskDescriptor>>,
  waiting: Queue<*mut TaskDescriptor>
}

pub struct Guard<'a>(&'a StaticMutex);

impl StaticMutex {
  // This is a bit subtle:
  // We need to add ourselves to the mutex's waiting list. To do this
  // we allocate a list item on the local stack, append it to the
  // waiting list, and block. When the task before us unlocks the mutex,
  // they will wake us up. Finally, when we are executing again we
  // remove our entry from the list.
  pub fn lock<'a>(&'a self) -> Guard<'a> {
    unsafe {
      let irq_disabled = disable_irqs();
      match self.owner.get() {
        None    => { },
        Some(_) => {
          let mut waiting = Node::new(Tasks.current_task() as *mut TaskDescriptor);
          Tasks.current_task().block();
          self.waiting.push(&mut waiting, irq_disabled);
          enable_irqs(irq_disabled);
          task_scheduler();
          let irq_disabled = disable_irqs();
          self.waiting.pop(irq_disabled);
        }
      }

      self.owner.set(Some(Tasks.current_task() as *mut TaskDescriptor));
      enable_irqs(irq_disabled);
    }

    Guard(self)
  }

  fn unlock(&self) {
    unsafe {
      let irq_disabled = disable_irqs();
      self.owner.set(None);
      match self.waiting.peek() {
        None => { },
        Some(nextTask) => {
          let mut task = *(*nextTask).data;
          task.unblock();
        }
      }
      enable_irqs(irq_disabled);
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for Guard<'a> {
  fn drop(&mut self) {
    let &Guard(ref mutex) = self;
    mutex.unlock();
  }
}
