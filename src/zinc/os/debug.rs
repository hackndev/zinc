// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <ben@smart-cactus.org>
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

//! Tracing for debugging.
//
// Use `set_backend`

pub use os::debug::internal::{set_backend, print, Token};

#[cfg(debug)]
mod internal {
  use core::option::{Some, None, Option};
  use core::ops::Drop;
  use core::mem::transmute;
  use drivers::chario::CharIO;

  static mut backend: Option<*const CharIO + 'static> = None;

  /// A token to ensure the life of the reference to the debugging output backend
  /// doesn't outlive the backend itself.
  #[must_use]
  pub struct Token<'a> {
    #[allow(dead_code)]
    hello: ()
  }

  #[unsafe_destructor]
  impl<'a> Drop for Token<'a> {
    fn drop(&mut self) {
      unsafe {
        backend = None;
      }
    }
  }

  /// Set the debugging output backend (mock)
  pub fn set_backend<'a>(b: &'a CharIO) -> Token<'a> {
    unsafe {
      backend = Some(transmute(b));
      Token { hello: () }
    }
  }

  /// Print debugging output backend (mock)
  pub fn print(s: &str) {
    unsafe {
      match backend {
        Some(b) => (*b).puts(s),
        None => {},
      }
    }
  }
}

#[cfg(not(debug))]
mod internal {
  use drivers::chario::CharIO;

  /// Set the debugging output backend (mock)
  pub fn set_backend(_: &CharIO) { }

  /// Print debugging output backend (mock)
  pub fn print(_: &str) { }

  /// A token to ensure the life of the reference to the debugging output backend
  /// doesn't outlive the backend itself.
  #[must_use]
  pub struct Token {
    #[allow(dead_code)]
    hello: ()
  }
}
