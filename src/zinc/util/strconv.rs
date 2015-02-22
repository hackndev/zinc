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

//! A simple integer-to-string conversion code with statically sized buffers.

use core::mem::uninitialized;

/// Convert an integer to a string.
pub fn itoa(val: u32, buf : &mut [u8], base: u32) {
  let mut rbuf : [u8; 32] = unsafe { uninitialized() };
  let mut myval : u32 = val;
  let mut idx: isize = 0;

  if myval == 0 {
    buf[0] = '0' as u8;
    return;
  }

  while myval > 0 {
    let digit : u8 = (myval % base) as u8;
    myval = myval / base;

    rbuf[idx as usize] = digit;

    idx += 1;
  }

  idx -= 1;

  let mut ridx = 0;

  while idx >= 0 {
    let charcode : u8 = ['0','1','2','3','4','5','6','7','8','9','a','b','c','d','e','f'][rbuf[idx as usize] as usize] as u8;

    buf[ridx] = charcode;

    idx -= 1;
    ridx += 1;
  }
}
