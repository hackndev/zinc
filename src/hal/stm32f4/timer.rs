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

use core::volatile_load;

#[no_split_stack]
pub fn timer_init() {

}

#[no_split_stack]
#[inline(always)]
pub fn wait(s: u32) {
  wait_us(s * 1000000);
}

#[no_split_stack]
#[inline(always)]
pub fn wait_ms(ms: u32) {
  wait_us(ms * 1000);
}

#[no_split_stack]
#[inline(always)]
pub fn wait_us(us: u32) {
  let mut j: u32=0;
  while j < 1000000+us {
    unsafe { volatile_load(&(0 as *mut u32)); }
    j += 1;
  }
}
