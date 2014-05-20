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

//! debug::port provides interface to output structured data over serial port.

use core::mem::{size_of, transmute};
use core::intrinsics::abort;

use drivers::chario::CharIO;
use hal::uart::{UART, UARTConf};

extern {
  fn memcpy(dest: *mut u8, src: *u8, n: int);
}

// TODO(farcaller): fix when size_of is avaliable in statics.
static SizeOfUART: uint = 64;

static mut uart_buf: [u8, ..SizeOfUART] = [0, ..SizeOfUART];

/// Initializes debug port with uart configuration.
///
/// This function must be called before any debug port use.
pub fn setup(conf: &UARTConf) {
  if SizeOfUART < size_of::<UART>() {
    unsafe { abort() };
  }

  let uart: UART = conf.setup();

  unsafe {
    let src_ptr: *u8 = transmute(&uart);
    let dst_ptr: *mut u8 = transmute(&uart_buf);
    memcpy(dst_ptr, src_ptr, size_of::<UART>() as int);
  }
}

/// Returns a CharIO corresponding to current debug port.
pub fn io() -> &CharIO {
  let uart: &UART = unsafe { transmute(&uart_buf) };

  uart as &CharIO
}
