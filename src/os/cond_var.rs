use hal::cortex_m3::sched::CritSection;
use lib::queue::{Queue, Node};
use std::option::{None, Some};
use os::task::{TaskDescriptor, Tasks};

pub struct CondVar {
  waiting: Queue<*mut TaskDescriptor>
}

impl CondVar {
  /// Wait on a condition variable
  pub fn wait<'a>(&'a self) {
    unsafe {
      let crit = CritSection::new();
      let mut waiting = Node::new(Tasks.current_task() as *mut TaskDescriptor);
      self.waiting.push(&mut waiting, &crit);
      Tasks.current_task().block(crit);
      let crit = CritSection::new();
      self.waiting.pop(&crit);
    }
  }

  /// Wake up a thread waiting on a condition variable
  pub fn signal<'a>(&'a self) {
    unsafe {
      let crit = CritSection::new();
      match self.waiting.pop(&crit) {
        None => { }
        Some(task) => {
          (*(*task).data).unblock(&crit);
        }
      }
    }
  }

  /// Wake up all threads waiting on a condition variable
  pub fn broadcast<'a>(&'a self) {
    unsafe {
      let crit = CritSection::new();
      loop {
        match self.waiting.pop(&crit) {
          None => break,
          Some(task) => (*(*task).data).unblock(&crit)
        }
      }
    }
  }
}
