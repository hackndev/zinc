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

macro_rules! ioreg_old(
  ($io:ident: $ty:ty, $($reg:ident),+) => (
    #[allow(non_snake_case)]
    #[derive(Copy)]
    pub struct $io {
      $(
        $reg: VolatileCell<$ty>,
      )+
    }
  )
);

macro_rules! reg_r(
  ($t:ident, $ty:ty, $getter_name:ident, $reg:ident) => (
    impl $t {
      #[allow(dead_code,non_snake_case)]
      #[inline(always)]
      pub fn $getter_name(&self) -> $ty {
        self.$reg.get()
      }
    }
  )
);

macro_rules! reg_w(
  ($t:ident, $ty:ty, $setter_name:ident, $reg:ident) => (
    impl $t {
      #[allow(dead_code,non_snake_case)]
      #[inline(always)]
      pub fn $setter_name(&self, val: $ty) {
        self.$reg.set(val);
      }
    }
  )
);

macro_rules! reg_rw(
  ($t:ident, $ty:ty, $getter_name:ident, $setter_name:ident, $reg:ident) => (
    reg_r!($t, $ty, $getter_name, $reg);
    reg_w!($t, $ty, $setter_name, $reg);
  )
);
