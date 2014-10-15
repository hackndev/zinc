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

//! The final object that gets linked into the binary. Provides trampilines
//! for stack management, scheduler, ISRs and other symbols with global
//! visibility.

#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;
extern crate app;

#[no_stack_check]
#[no_mangle]
pub extern fn main() {
  app::main();
}

#[no_stack_check]
#[no_mangle]
#[cfg(not(cfg_multitasking))]
pub extern fn __morestack() {
  unsafe { core::intrinsics::abort() };
}

#[no_stack_check]
#[no_mangle]
#[cfg(cfg_multitasking)]
pub extern fn __morestack() {
  zinc::os::task::morestack();
}

#[no_stack_check]
#[no_mangle]
#[cfg(cfg_multitasking)]
pub unsafe fn task_scheduler() {
  zinc::os::task::task_scheduler();
}
