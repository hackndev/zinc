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

//! A cell that with volatile setter and getter.

#![feature(no_std, core_intrinsics)]
#![no_std]


#[cfg(feature="replayer")] #[macro_use(expect)] extern crate expectest;
#[cfg(feature="replayer")] #[macro_use] extern crate std;

#[cfg(feature="replayer")] use std::vec::Vec;
#[cfg(feature="replayer")] use expectest::prelude::*;
#[cfg(feature="replayer")] use std::string::String;
#[cfg(feature="replayer")] use std::fmt;
#[cfg(feature="replayer")] use core::cmp::PartialEq;
#[cfg(feature="replayer")] use core::clone::Clone;
#[cfg(feature="replayer")] use core::cell::RefCell;

#[cfg(not(feature="replayer"))] use core::intrinsics::{volatile_load, volatile_store};
#[cfg(feature="replayer")] use core::intrinsics::transmute;

// TODO(farcaller): why this needs copy/clone?
/// This structure is used to represent a hardware register.
/// It is mostly used by the ioreg family of macros.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VolatileCell<T> {
  value: T,
}

impl<T> VolatileCell<T> {
  /// Create a cell with initial value.
  pub fn new(value: T) -> VolatileCell<T> {
    VolatileCell {
      value: value,
    }
  }

  /// Get register value.
  #[cfg(not(feature="replayer"))]
  #[inline]
  pub fn get(&self) -> T {
    unsafe {
      volatile_load(&self.value)
    }
  }

  /// Set register value.
  #[cfg(not(feature="replayer"))]
  #[inline]
  pub fn set(&self, value: T) {
    unsafe {
      volatile_store(&self.value as *const T as *mut T, value)
    }
  }
}

#[cfg(feature="replayer")]
impl VolatileCell<u32> {
  pub fn get(&self) -> u32 {
    unsafe {
      GLOBAL_REPLAYER.with(|gr| { gr.borrow_mut().get_cell(transmute(&self.value)) })
    }
  }

  pub fn set(&self, value: u32) {
    unsafe {
      GLOBAL_REPLAYER.with(|gr| { gr.borrow_mut().set_cell(transmute(&self.value), value) })
    }
  }
}

#[cfg(feature="replayer")]
impl VolatileCell<u16> {
  pub fn get(&self) -> u16 {
    unsafe {
      GLOBAL_REPLAYER.with(|gr| { gr.borrow_mut().get_cell(transmute(&self.value)) }) as u16
    }
  }

  pub fn set(&self, value: u16) {
    unsafe {
      GLOBAL_REPLAYER.with(|gr| { gr.borrow_mut().set_cell(transmute(&self.value), value as u32) })
    }
  }
}

#[cfg(feature="replayer")]
impl VolatileCell<u8> {
  pub fn get(&self) -> u8 {
    unsafe {
      GLOBAL_REPLAYER.with(|gr| { gr.borrow_mut().get_cell(transmute(&self.value)) }) as u8
    }
  }

  pub fn set(&self, value: u8) {
    unsafe {
      GLOBAL_REPLAYER.with(|gr| { gr.borrow_mut().set_cell(transmute(&self.value), value as u32) })
    }
  }
}

#[cfg(feature="replayer")]
struct ReplayRecord {
  is_read: bool,
  address: usize,
  value: u32,

  replayed: bool,
  did_read: bool,
  actual_address: usize,
  actual_value: u32,

  loc: expectest::core::SourceLocation,
}

#[cfg(feature="replayer")]
impl core::fmt::Display for ReplayRecord {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    match self.is_read {
      true  => write!(f, "read 0x{:x} from 0x{:x}", self.value, self.address),
      false => write!(f, "write 0x{:x} to 0x{:x}", self.value, self.address),
    }
  }
}

#[cfg(feature="replayer")]
pub struct VolatileCellReplayer {
  replays: Vec<ReplayRecord>,
  current_replay: usize,
}

#[cfg(feature="replayer")]
impl VolatileCellReplayer {
  pub fn new() -> VolatileCellReplayer {
    VolatileCellReplayer {
      replays: Vec::new(),
      current_replay: 0,
    }
  }

  pub fn expect_read(&mut self, address: usize, value: u32,
      loc: expectest::core::SourceLocation) {
    self.replays.push(ReplayRecord {
      is_read: true,
      address: address,
      value: value,
      replayed: false,
      did_read: false,
      actual_address: 0,
      actual_value: 0,
      loc: loc,
    });
  }

  pub fn expect_write(&mut self, address: usize, value: u32,
      loc: expectest::core::SourceLocation) {
    self.replays.push(ReplayRecord {
      is_read: false,
      address: address,
      value: value,
      replayed: false,
      did_read: false,
      actual_address: 0,
      actual_value: 0,
      loc: loc,
    });
  }

  pub fn verify(&self, loc: expectest::core::SourceLocation) {
    expect(self.current_replay).location(loc).to(
      be_equal_to_with_context(
          self.replays.len(),
          format!("expected {} replays, performed {}",
              self.replays.len(), self.current_replay)));

    for ref replay in &*self.replays {
      expect(replay.replayed).location(replay.loc).to(be_equal_to_with_context(true,
        format!("expected replay {} to be performed, was not", replay)));
      expect(replay.is_read).location(replay.loc).to(be_equal_to_with_context(replay.did_read,
        format!("expected replay to be {} replay, was {} replay",
          if replay.is_read {"read"} else {"write"},
          if replay.is_read {"write"} else {"read"})));
      expect(replay.address).location(replay.loc).to(be_equal_to_with_context(replay.actual_address,
        format!("expected replay address 0x{:x}, was 0x{:x}", replay.address, replay.actual_address)));
      if !replay.is_read {
        expect(replay.value).location(replay.loc).to(be_equal_to_with_context(replay.actual_value,
          format!("expected replay to write 0x{:x}, written 0x{:x}", replay.value, replay.actual_value)));
      }
    }
  }

  pub fn get_cell(&mut self, address: usize) -> u32 {
    if self.current_replay >= self.replays.len() {
      panic!("get_cell(0x{:x}) faled, current replay: {}, total replays: {}",
        address, self.current_replay+1, self.replays.len());
    }
    let replay: &mut ReplayRecord = &mut self.replays[self.current_replay];
    replay.replayed = true;
    replay.did_read = true;
    replay.actual_address = address;

    self.current_replay += 1;

    replay.value
  }

  pub fn set_cell(&mut self, address: usize, value: u32) {
    if self.current_replay >= self.replays.len() {
      panic!("set_cell(0x{:x}, 0x{:x}) faled, current replay: {}, total replays: {}",
        address, value, self.current_replay+1, self.replays.len());
    }
    let replay: &mut ReplayRecord = &mut self.replays[self.current_replay];
    replay.replayed = true;
    replay.did_read = false;
    replay.actual_address = address;
    replay.actual_value = value;

    self.current_replay += 1;
  }
}

#[cfg(feature="replayer")]
thread_local!(static GLOBAL_REPLAYER: RefCell<VolatileCellReplayer> = RefCell::new(VolatileCellReplayer::new()));

#[cfg(feature="replayer")]
pub fn set_replayer(replayer: VolatileCellReplayer) {
  GLOBAL_REPLAYER.with(|gr| {
    let mut bm = gr.borrow_mut();
    *bm = replayer;
  });
}

#[cfg(feature="replayer")]
pub fn with_mut_replayer<F>(f: F) where F: core::ops::FnOnce(&mut VolatileCellReplayer) {
  GLOBAL_REPLAYER.with(|gr| {
    let mut bm = gr.borrow_mut();
    f(&mut *bm);
  });
}

#[cfg(feature="replayer")]
struct BeEqualToWithContext<E> {
    expected: E,
    context: String,
}

#[cfg(feature="replayer")]
fn be_equal_to_with_context<E>(expected: E, context: String) -> BeEqualToWithContext<E> {
    BeEqualToWithContext {
        expected: expected,
        context: context,
    }
}

#[cfg(feature="replayer")]
impl<A, E> Matcher<A, E> for BeEqualToWithContext<E>
    where
        A: PartialEq<E> + fmt::Debug,
        E: fmt::Debug {

    fn failure_message(&self, _: expectest::core::Join, _: &A) -> String {
        self.context.clone()
    }

    fn matches(&self, actual: &A) -> bool {
        *actual == self.expected
    }
}

#[macro_export]
macro_rules! expect_volatile_read {
  ($addr: expr, $val: expr) => (
    $crate::with_mut_replayer(|r| {
      r.expect_read($addr, $val, expectest::core::SourceLocation::new(file!(), line!()));
    })
  );
}

#[macro_export]
macro_rules! expect_volatile_write {
  ($addr: expr, $val: expr) => (
    $crate::with_mut_replayer(|r| {
      r.expect_write($addr, $val, expectest::core::SourceLocation::new(file!(), line!()));
    })
  );
}

#[macro_export]
macro_rules! expect_replayer_valid {
  () => (
    $crate::with_mut_replayer(|r| {
      r.verify(expectest::core::SourceLocation::new(file!(), line!()));
    })
  );
}

#[macro_export]
macro_rules! init_replayer {
  () => (
    set_replayer(VolatileCellReplayer::new());
  );
}
