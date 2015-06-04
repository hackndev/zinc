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

#![feature(core, no_std)]
#![no_std]

#[cfg(feature="replayer")] extern crate hamcrest;
#[cfg(feature="replayer")] #[macro_use] extern crate std;

#[cfg(feature="replayer")] use std::vec::Vec;
#[cfg(feature="replayer")] use hamcrest::{assert_that, is, equal_to};

extern crate core;

#[cfg(not(feature="replayer"))] use core::intrinsics::{volatile_load, volatile_store};
#[cfg(feature="replayer")] use core::intrinsics::transmute;

// TODO(farcaller): why this needs copy/clone?
/// This structure is used to represent a hardware register.
/// It is mostly used by the ioreg family of macros.
#[derive(Copy, Clone)]
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
      (*GlobalReplayer).get_cell(transmute(&self.value))
    }
  }

  pub fn set(&self, value: u32) {
    unsafe {
      (*GlobalReplayer).set_cell(transmute(&self.value), value)
    }
  }
}

#[cfg(feature="replayer")]
impl VolatileCell<u16> {
  pub fn get(&self) -> u16 {
    unsafe {
      (*GlobalReplayer).get_cell(transmute(&self.value)) as u16
    }
  }

  pub fn set(&self, value: u16) {
    unsafe {
      (*GlobalReplayer).set_cell(transmute(&self.value), value as u32)
    }
  }
}

#[cfg(feature="replayer")]
impl VolatileCell<u8> {
  pub fn get(&self) -> u8 {
    unsafe {
      (*GlobalReplayer).get_cell(transmute(&self.value)) as u8
    }
  }

  pub fn set(&self, value: u8) {
    unsafe {
      (*GlobalReplayer).set_cell(transmute(&self.value), value as u32)
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

  pub fn expect_read(&mut self, address: usize, value: u32) {
    self.replays.push(ReplayRecord {
      is_read: true,
      address: address,
      value: value,
      replayed: false,
      did_read: false,
      actual_address: 0,
      actual_value: 0,
    });
  }

  pub fn expect_write(&mut self, address: usize, value: u32) {
    self.replays.push(ReplayRecord {
      is_read: false,
      address: address,
      value: value,
      replayed: false,
      did_read: false,
      actual_address: 0,
      actual_value: 0,
    });
  }

  pub fn verify(&self) {
    assert_that(self.current_replay, is(equal_to(self.replays.len())));

    let mut i = 1usize;
    for ref replay in &*self.replays {
      println!("replay {}", i);
      println!("replayed?");
      assert_that(replay.replayed, is(equal_to(true)));
      println!("is read?");
      assert_that(replay.is_read, is(equal_to(replay.did_read)));
      println!("address correct?");
      assert_that(replay.address, is(equal_to(replay.actual_address)));
      if !replay.is_read {
        println!("value written is correct?");
        assert_that(replay.value, is(equal_to(replay.actual_value)));
      }
      i += 1;
    }
  }

  pub fn get_cell(&mut self, address: usize) -> u32 {
    if self.current_replay >= self.replays.len() {
      panic!("get_cell({}) faled, current replay: {}, total replays: {}",
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
      panic!("set_cell({}, {}) faled, current replay: {}, total replays: {}",
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
static mut GlobalReplayer: *mut VolatileCellReplayer = 0 as *mut VolatileCellReplayer;

#[cfg(feature="replayer")]
pub fn set_replayer(replayer: &mut VolatileCellReplayer) {
  unsafe {
    GlobalReplayer = replayer;
  }
}
