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

#![feature(macro_rules)]
#![crate_id="zinc_macros"]
#![crate_type="rlib"]
#![no_std]

#[macro_export]
macro_rules! route_isr(
  ($name:ident -> $dest:expr) => (
    #[no_mangle]
    #[no_split_stack]
    #[inline(never)]
    pub unsafe fn $name() {
      $dest();
    }
  )
)
