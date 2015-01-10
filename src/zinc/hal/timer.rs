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
Timer interface.

TimerConf is a MCU-specific struct.

Timers provide a simple way to delay program execution for some time.
*/

#[path="../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Timer implementation.
pub trait Timer {
  /// Implementation-specific method to wait a given number of microseconds.
  fn get_counter(&self) -> u32;

  #[inline(always)]
  /// Waits for specified number of microseconds.
  fn wait_us(&self, us: u32) {
    let start = self.get_counter();
    wait_for!((self.get_counter() - start) >= us);
  }

  #[inline(always)]
  /// Waits for specified number of milliseconds.
  fn wait_ms(&self, ms: u32) {
    self.wait_us(ms * 1000);
  }

  #[inline(always)]
  /// Waits for specified number of seconds.
  fn wait(&self, s: u32) {
    self.wait_us(s * 1000000);
  }
}
