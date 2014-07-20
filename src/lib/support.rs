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

#![no_std]
#![crate_type="rlib"]
#![feature(asm, intrinsics)]

#[cfg(test)]
#[no_split_stack]
#[no_mangle]
pub fn breakpoint() { unimplemented!() }

#[cfg(not(test))]
#[no_split_stack]
#[no_mangle]
pub fn breakpoint() {
  unsafe { asm!("bkpt") }
}

#[no_split_stack]
#[no_mangle]
pub fn abort() -> ! {
  breakpoint();
  loop {}
}

#[no_split_stack]
#[no_mangle]
pub fn __aeabi_unwind_cpp_pr0() {
  abort();
}

#[no_split_stack]
#[no_mangle]
pub fn __aeabi_unwind_cpp_pr1() {
  abort();
}

#[no_split_stack]
#[no_mangle]
pub fn get_eit_entry() {
  abort();
}

#[no_split_stack]
#[no_mangle]
pub fn unwind_phase2_forced() {
  abort();
}

#[no_split_stack]
#[no_mangle]
pub fn unwind_phase2() {
  abort();
}

#[no_split_stack]
#[no_mangle]
pub unsafe fn rust_fail_bounds_check() {
  abort();
}
#[no_split_stack]
#[no_mangle]
pub fn rust_begin_unwind() {
  abort();
}
