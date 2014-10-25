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

/*!
Basic round-robin scheduler.

TODO(farcaller): it's not round-robin, actually. A stricter time slice
                 accounting must be done.
*/

use core::collections::Collection;

use super::task;
use super::stack::StackManager;
use hal::systick::Systick;

/// Scheduler interface.
pub struct Scheduler<'a, T, S> {
  index: task::TasksIndex<'a>,
  context_switch: ||:'a,
  systick: T,
  stack_manager: S,
}

impl<'a, T: Systick, S: StackManager> Scheduler<'a, T, S> {
  /// Creates a new scheduler given a list of tasks, systick timer and
  /// management routines.
  ///
  /// At least one task must be defined in task index.
  pub fn new(ti: task::TasksIndex<'a>, systick: T, stack_manager: S,
    ctx_switch: ||:'a) -> Scheduler<'a, T, S> {
    Scheduler {
      index: ti,
      context_switch: ctx_switch,
      systick: systick,
      stack_manager: stack_manager,
    }
  }

  /// Starts a scheduler and switches to first task. Never returns.
  pub fn start(&mut self) {
    self.stack_manager.set_task_stack_pointer(
        self.index.tasks[self.index.current_task_index as uint].stack_start);
    self.systick.start();
    (self.context_switch)();
  }

  /// Switches to next task.
  ///
  /// Intended to be run by systick ISR, not invoked directly.
  pub fn switch(&mut self) {
    self.index.tasks[self.index.current_task_index as uint].stack_start =
        self.stack_manager.get_task_stack_pointer();

    self.index.current_task_index += 1;
    if (self.index.current_task_index as uint) == self.index.tasks.len() {
      self.index.current_task_index = 0;
    }

    self.stack_manager.set_task_stack_pointer(
        self.index.tasks[self.index.current_task_index as uint].stack_start);
  }

  fn current_task_index(&self) -> u8 {
    self.index.current_task_index
  }

  fn index(&self) -> &task::TasksIndex {
    &self.index
  }
}

#[cfg(test)]
mod test {
  use hamcrest::{assert_that, is, equal_to};
  use std::kinds::marker;

  use hal::systick::Systick;
  use os::sched::stack::StackManager;
  use os::sched::task;
  use super::Scheduler;

  struct FakeSystick {
    started_ptr: *mut bool
  }

  impl FakeSystick {
    pub fn new(started: &mut bool) -> FakeSystick {
      FakeSystick {
        started_ptr: started as *mut bool
      }
    }
  }
  impl Systick for FakeSystick {
    fn start(&self) {
      unsafe { *self.started_ptr = true; }
    }
  }

  struct FakeStackManager {
    pub sp_ptr: *mut u32
  }
  impl FakeStackManager {
    pub fn new(sp: &mut u32) -> FakeStackManager {
      FakeStackManager {
        sp_ptr: sp as *mut u32
      }
    }
  }
  impl StackManager for FakeStackManager {
    fn get_task_stack_pointer(&self) -> u32 {
      unsafe { *self.sp_ptr }
    }
    fn set_task_stack_pointer(&self, sp: u32) {
      unsafe { *self.sp_ptr = sp; }
    }
  }

  describe!(
    before_each {
      let mut systick_started = false;
      let tick = FakeSystick::new(&mut systick_started);
      let mut tasks = [task::Task {
        state: task::Runnable,
        stack_start: 100,
        stack_end: 200,
      },
      task::Task {
        state: task::Runnable,
        stack_start: 200,
        stack_end: 300,
      }];
      let ti = task::TasksIndex {
        tasks: tasks,
        current_task_index: 0,
        no_copy: marker::NoCopy,
      };
      let mut sp = 0u32;
      let fsm = FakeStackManager::new(&mut sp);
    }

    it "calls a context switch with first task" {
      let mut called = false;

      {
        let mut scheduler = Scheduler::new(ti, tick, fsm, || { called = true });
        scheduler.start();
      }

      assert_that(called, is(equal_to(true)));
    }

    it "schedules second task on timer interrupt" {
      let mut scheduler = Scheduler::new(ti, tick, fsm, || {});
      scheduler.start();

      scheduler.switch();

      assert_that(scheduler.current_task_index(), is(equal_to(1u8)));
    }

    it "wraps over to first task when all tasks are done" {
      let mut scheduler = Scheduler::new(ti, tick, fsm, || {});
      scheduler.start();

      scheduler.switch();
      scheduler.switch();

      assert_that(scheduler.current_task_index(), is(equal_to(0u8)));
    }

    it "enables systick timer on start" {
      let mut scheduler = Scheduler::new(ti, tick, fsm, || {});
      scheduler.start();

      assert_that(systick_started, is(equal_to(true)));
    }

    it "loads first task stack pointer" {
      let mut scheduler = Scheduler::new(ti, tick, fsm, || {});
      scheduler.start();

      assert_that(sp, is(equal_to(100u32)));
    }

    it "saves stack pointer to current task on switch" {
      let mut scheduler = Scheduler::new(ti, tick, fsm, || {});
      scheduler.start();

      sp = 110;
      scheduler.switch();

      assert_that(scheduler.index().tasks[0].stack_start, is(equal_to(110u32)));
      assert_that(sp, is(equal_to(200u32)));
    }

    it "loads stack pointer to next task on switch" {
      let mut scheduler = Scheduler::new(ti, tick, fsm, || {});
      scheduler.start();

      scheduler.switch();

      assert_that(sp, is(equal_to(200u32)));
    }
  )
}
