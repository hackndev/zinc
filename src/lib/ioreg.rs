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

#![macro_escape]

macro_rules! ioreg(
  ($io:ident: $($reg:ident),+) => (
    #[allow(uppercase_variables)]
    pub struct $io {
      $(
        $reg: VolatileCell<u32>,
      )+
    }
  )
)

macro_rules! reg_r(
  ($t:ident, $getter_name:ident, $reg:ident) => (
    impl $t {
      #[no_split_stack]
      #[allow(dead_code,non_snake_case_functions)]
      #[inline(always)]
      pub fn $getter_name(&self) -> u32 {
        self.$reg.get()
      }
    }
  )
)

macro_rules! reg_w(
  ($t:ident, $setter_name:ident, $reg:ident) => (
    impl $t {
      #[no_split_stack]
      #[allow(dead_code,non_snake_case_functions)]
      #[inline(always)]
      pub fn $setter_name(&self, val: u32) {
        self.$reg.set(val);
      }
    }
  )
)

macro_rules! reg_rw(
  ($t:ident, $getter_name:ident, $setter_name:ident, $reg:ident) => (
    reg_r!($t, $getter_name, $reg)
    reg_w!($t, $setter_name, $reg)
  )
)
