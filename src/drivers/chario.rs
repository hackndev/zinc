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

//! Generic char output trait.

use core::option::{Some, None};
use core::str::{Str, StrSlice};
use core::slice::{Vector, ImmutableVector};
use core::container::Container;
use core::iter::Iterator;

use core::mem::zeroed;

use lib::strconv;

/// CharIO provides interface for outputting characters.
///
/// This trait implements the common functions to output strings and numbers,
/// requiring only one method: `putc`.
pub trait CharIO {
  /// Outputs a character.
  fn putc(&self, value: char);

  /// Outputs a string.
  fn puts(&self, s: &str) {
    let chars : &[u8] = s.as_slice().as_bytes();
    let mut i = 0;
    while i < s.len() {
      let c : char = chars[i] as char;
      self.putc(c);
      i += 1;
    }
  }

  /// Outputs an integer with given base.
  fn putint(&self, i: u32, base: u32) {
    let mut buf : [u8, ..32] = unsafe { zeroed() };
    let bsl : &mut [u8] = buf;
    strconv::itoa(i, bsl, base);

    for &i in bsl.iter() {
      if i == 0 {
        break;
      }
      self.putc(i as char);
    }
  }

  /// Outputs an integer.
  fn puti(&self, i: u32) {
    self.putint(i, 10);
  }

  /// Outputs an integer as a hex string.
  fn puth(&self, i: u32) {
    self.putint(i, 16);
  }
}
