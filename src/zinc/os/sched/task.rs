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

//! Task structures.

use core::kinds::marker;

/// Task states.
pub enum State {
  /// This task can be scheduled.
  Runnable
}

/// Task descriptor. Should be treated as opaque struct.
pub struct Task {
  /// Current task state.
  pub state: State,

  /// Pointer to top of the stack.
  pub stack_start: u32,

  /// Pointer to the lowest possible stack address.
  pub stack_end: u32,
}

/// Tasks index provides a scheduler with a list of all registered tasks and
/// initial state.
#[packed]
pub struct TasksIndex<'a> {
  /// A mutabler slice with all defined tasks.
  pub tasks: &'a mut [Task],

  /// Current running task index.
  pub current_task_index: u8,

  /// Tasks are not copyable.
  pub no_copy: marker::NoCopy,
}
