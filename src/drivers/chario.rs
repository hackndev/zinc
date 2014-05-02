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

use lib::strconv;
use core::str;
use core::init;
use core::option::{Some, None};
use core::iter::Iterator;
use core::slice::iter;
use core::container::Container;

/// CharIO provides interface for outputting characters.
///
/// This trait implements the common functions to output strings and numbers,
/// requiring only one method: `putc`.
pub trait CharIO {
  /// Outputs a character.
  fn putc(&self, value: char);

  /// Outputs a string.
  fn puts(&self, s: &str) {
    let chars : &[u8] = str::as_bytes(s);
    let mut i = 0;
    while i < s.len() {
      let c : char = chars[i] as char;
      self.putc(c);
      i += 1;
    }
  }

  /// Outputs an integer with given base.
  fn putint(&self, i: u32, base: u32) {
    let mut buf : [u8, ..32] = unsafe { init() };
    let bsl : &mut [u8] = buf;
    strconv::itoa(i, bsl, base);

    for &i in iter(bsl) {
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
