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

use core::slice::SliceExt;
use core::convert::AsRef;

use core::mem::zeroed;

use util::strconv;

/// CharIO provides interface for outputting characters.
///
/// This trait implements the common functions to output strings and numbers,
/// requiring only one method: `putc`.
pub trait CharIO {
  /// Outputs a character.
  fn putc(&self, value: char);

  /// Outputs a string.
  fn puts(&self, s: &str) {
    let chars : &[u8] = s.as_ref();
    for i in 0 .. chars.len() {
      let c : char = chars[i] as char;
      self.putc(c);
    }
  }

  /// Outputs an integer with given base.
  fn putint(&self, i: u32, base: u32) {
    let mut buf : [u8; 32] = unsafe { zeroed() };
    let bsl : &mut [u8] = &mut buf;
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

#[cfg(test)]
pub mod test {
  use core::cell::RefCell;

  use drivers::chario::CharIO;

  #[derive(Clone, Copy)]
  pub struct TestCharIOData {
    last_char: char,
    putc_calls: usize,
  }

  pub struct TestCharIO {
    data: RefCell<TestCharIOData>
  }

  impl CharIO for TestCharIO {
    fn putc(&self, value: char) {
      let mut data = self.data.borrow_mut();
      data.putc_calls += 1;
      data.last_char = value;
    }
  }

  impl TestCharIO {
    pub fn new() -> TestCharIO {
      TestCharIO {
        data: RefCell::new(TestCharIOData {
          last_char: '\0',
          putc_calls: 0,
        }),
      }
    }

    fn get_last_char(&self) -> char {
      self.data.borrow().last_char
    }

    fn get_and_reset_putc_calls(&self) -> usize {
      let current = self.data.borrow().putc_calls;
      self.data.borrow_mut().putc_calls = 0;
      current
    }
  }

  #[test]
  fn putc_should_store_a_char() {
    let io = TestCharIO::new();
    io.putc('a');
    assert!(io.get_last_char() == 'a');
    io.putc('z');
    assert!(io.get_last_char() == 'z');
  }

  #[test]
  fn puti_should_store_a_number_as_char() {
    let io = TestCharIO::new();
    io.puti(3);
    assert!(io.get_last_char() == '3');
    io.puti(9);
    assert!(io.get_last_char() == '9');
    io.puti(10);
    assert!(io.get_last_char() == '0');
    io.puti(11);
    assert!(io.get_last_char() == '1');
  }

  #[test]
  fn puth_should_store_a_number_as_char() {
    let io = TestCharIO::new();
    io.puth(3);
    assert!(io.get_last_char() == '3');
    io.puth(9);
    assert!(io.get_last_char() == '9');
    io.puth(10);
    assert!(io.get_last_char() == 'a');
    io.puth(11);
    assert!(io.get_last_char() == 'b');
    io.puth(16);
    assert!(io.get_last_char() == '0');
    io.puth(17);
    assert!(io.get_last_char() == '1');
  }

  #[test]
  fn putint_should_work_with_different_bases() {
    let io = TestCharIO::new();
    io.putint(0, 2);
    assert!(io.get_last_char() == '0');
    io.putint(1, 2);
    assert!(io.get_last_char() == '1');
    io.putint(2, 2);
    assert!(io.get_last_char() == '0');
    io.putint(3, 2);
    assert!(io.get_last_char() == '1');
    io.putint(7, 7);
    assert!(io.get_last_char() == '0');
    io.putint(8, 7);
    assert!(io.get_last_char() == '1');
    io.putint(12, 7);
    assert!(io.get_last_char() == '5');
    io.putint(14, 7);
    assert!(io.get_last_char() == '0');
  }

  #[test]
  fn puts_should_leave_us_with_just_the_last_char() {
    let io = TestCharIO::new();
    io.puts("fu!");
    assert!(io.get_last_char() == '!');
    assert!(io.get_and_reset_putc_calls() == 3);
    io.puts("\n\t");
    assert!(io.get_last_char() == '\t');
    assert!(io.get_and_reset_putc_calls() == 2);
  }
}
