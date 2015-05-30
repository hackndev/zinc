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

//! Basic multitasking interface.

use core::mem::size_of;
use core::intrinsics::abort;

use hal::cortex_m3::{sched, systick};
use hal::cortex_m3::sched::NoInterrupts;
use os::syscall::syscall;
use hal::stack;

/// Task takes one argument, which is u32.
pub type Task = fn(u32);

mod current_stack_offset {
  /// Currently allocated stack memory, growing down, starting at __STACK_BASE.
  static mut CurrentStackOffset: u32 = 0;

  pub fn get() -> u32 {
    unsafe { CurrentStackOffset }
  }

  pub fn set(val: u32) {
    unsafe { CurrentStackOffset = val };
  }
}

/// Bytes to reserve in privileged stack based on stack size at the time of task::setup() call.
static ReservedPivilegedStackSize: u32 = 256;

/// Maximum number of tasks.
static MaxTasksCount: usize = 4;

mod defined_tasks_count {
  use core::intrinsics::abort;

  /// Total defined tasks count.
  static mut DefinedTasksCount: usize = 0;

  pub fn get() -> usize {
    unsafe { DefinedTasksCount }
  }

  pub fn increase() {
    unsafe {
      DefinedTasksCount += 1;
      if DefinedTasksCount > super::MaxTasksCount {
        abort();
      }
    }
  }
}

pub enum Status {
  Runnable,
  Blocked
}

/// Task descriptor, provides task stack pointer.
pub struct TaskDescriptor {
  pub stack_start: u32,
  pub stack_end: u32,
  pub status: Status
}

impl TaskDescriptor {
  pub fn block(&mut self, _: NoInterrupts) {
    self.status = Blocked;
    sched::switch_context();
  }
  pub fn unblock(&mut self, _: &NoInterrupts) { self.status = Runnable; }
}

struct TasksCollection {
  pub current_task: usize,
  pub tasks: [TaskDescriptor; MaxTasksCount],
}

pub static mut Tasks: TasksCollection = TasksCollection {
  current_task: 0,
  tasks: [TaskDescriptor { stack_start: 0, stack_end: 0, status: Runnable }; MaxTasksCount]
};

impl TasksCollection {
  pub fn current_task<'a>(&'a mut self) -> &'a mut TaskDescriptor {
    &mut self.tasks[self.current_task]
  }

  fn next_task(&mut self) {
    loop {
      self.current_task += 1;
      if self.current_task == defined_tasks_count::get() {
        self.current_task = 0;
      }
      match self.current_task() {
        &task if !task.valid()                 => {}
        &TaskDescriptor {status: Runnable, ..} => break,
        _                                      => {}
      }
    }
  }

  fn add_task(&mut self, t: TaskDescriptor) {
    self.tasks[defined_tasks_count::get()] = t;
    defined_tasks_count::increase();
  }
}

/// Initialize and start task manager.
///
/// This function keeps main stack intact. It starts the task scheduler and
/// never returns.
///
/// t should point to initial task.
#[inline(never)]
pub fn setup(t: Task, stack_size: u32) {
  systick::setup(::hal::cortex_m3::systick::CALIBRATED, true);

  let current_stack = sched::get_current_stack_pointer();
  // User tasks start at this current stack size + reserved size aligned by 4
  // bytes.
  let task_stack_base: u32 = (current_stack as u32 - ReservedPivilegedStackSize) & !3;
  current_stack_offset::set(task_stack_base);

  let td = define_task(t, 0, stack_size, true);

  td.load();

  systick::enable();
  sched::switch_context();

  unsafe { abort() };
}

#[inline(never)]
pub fn define_task(t: Task, arg: u32, stack_size: u32, initial: bool) -> TaskDescriptor {
  systick::disable_irq();
  let task_base = current_stack_offset::get();
  let task_stack_size: u32 = (
    stack_size +
    8*4 +  // hw saved regs
    8*4 +  // sw saved regs
    8*4    // scratch pad for __morestack failure. see note on morestack below.
  ) & !0b1111;
  current_stack_offset::set(task_base - task_stack_size);

  let td = TaskDescriptor::new(t, arg, task_base, stack_size, initial);
  unsafe { Tasks.add_task(td) };

  systick::enable_irq();
  td
}

impl TaskDescriptor {
  /// Creates a new TaskDescriptor for given task, arg and stack base.
  ///
  /// This function initializes task stack with hw saved registers.
  #[inline(never)]
  pub fn new(t: Task, arg: u32, stack_base: u32, stack_size: u32, initial: bool) -> TaskDescriptor {
    let state = sched::SavedState::new(t, arg);

    let mut stack_top: u32 = stack_base - size_of::<sched::SavedState>() as u32;
    unsafe { *(stack_top as *mut sched::SavedState) = state };
    if !initial {
      stack_top -= 8*4;
    }

    TaskDescriptor {
      stack_start: stack_top,
      stack_end: stack_base - stack_size,
      status: Runnable,
    }
  }

  pub fn load(&self) {
    sched::set_task_stack_pointer(self.stack_start);
    stack::set_stack_limit(self.stack_end);
  }

  pub fn save(&mut self) {
    self.stack_start = sched::get_task_stack_pointer();
  }

  pub fn valid(&self) -> bool {
    self.stack_end != 0
  }

  pub fn invalidate(&mut self) {
    self.stack_end = 0;
  }
}

#[inline(always)]
pub unsafe fn task_scheduler() {
  stack::set_stack_limit(stack::stack_base() - ReservedPivilegedStackSize);
  Tasks.current_task().save();
  Tasks.next_task();
  Tasks.current_task().load();
}

// TODO(farcaller): this should not actually use stack!
// At the time of the call of syscall(), the stack is overflown by 4, we still
// have 12 bytes in reserve and 2*8*4 to save the frame in pendsv after kill.
#[no_stack_check]
pub fn morestack() {
  let psp = sched::get_task_stack_pointer();
  let sp = sched::get_current_stack_pointer();
  if psp == sp {
    unsafe { syscall(kill_current_task, 0) };
  } else {
    unsafe { abort() };
  }
}

#[inline(never)]
#[no_mangle]
#[no_stack_check]
pub fn kill_current_task(_: u32) {
  unsafe { Tasks.current_task().invalidate() };
  sched::switch_context();
}
