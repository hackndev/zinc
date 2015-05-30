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

//! Driver for DHT22.

use core::option::Option::{self, Some, None};

use hal::pin::Gpio;
use hal::pin::GpioLevel::Low;
use hal::pin::GpioLevel::High;
use hal::pin::GpioDirection::In;
use hal::pin::GpioDirection::Out;
use hal::pin::GpioLevel;
use hal::timer::Timer;

/// Basic DHT22 driver ported over from Arduino example.
pub struct DHT22<'a, T:'a, P:'a> {
  gpio: &'a P,
  timer: &'a T,
}

/// Measurement data from the DHT22.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Measurements {
  pub humidity: f32,
  pub temperature: f32,
}

impl<'a, T: Timer, P: Gpio> DHT22<'a, T, P> {
  /// Creates a new DHT22 driver based on I/O GPIO and a timer with 10us resolution.
  pub fn new(timer: &'a T, gpio: &'a P) -> DHT22<'a, T, P> {
    DHT22 {
      gpio: gpio,
      timer: timer,
    }
  }

  /// Returns previous sensor measurements or None if synchronization failed.
  pub fn read(&self) -> Option<Measurements> {
    let buffer: &mut [u8; 5] = &mut [0; 5];
    let mut idx: usize = 0;
    let mut mask: u8 = 128;

    self.gpio.set_direction(Out);
    self.gpio.set_low();
    self.timer.wait_ms(20);
    self.gpio.set_high();
    self.timer.wait_us(40);
    self.gpio.set_direction(In);

    if !self.wait_sync() {
      return None
    }

    for _ in 0..40 {
      if !self.wait_while(Low, 80) {
        return None
      }

      let t = self.timer.get_counter();

      if !self.wait_while(High, 80) {
        return None
      }

      if self.timer.get_counter() - t > 40 {
        buffer[idx] |= mask;
      }

      mask >>= 1;
      if mask == 0 {
        mask = 128;
        idx += 1;
      }
    }

    let humidity: f32 = (((buffer[0] as u16) << 8) | buffer[1] as u16) as f32 * 0.1;
    let temperature: f32 = if buffer[2] & 0x80 != 0 {
      -0.1 * (((buffer[2] as u16 & 0x7F) << 8) | buffer[3] as u16) as f32
    } else {
      0.1 * (((buffer[2] as u16) << 8) | buffer[3] as u16) as f32
    };
    let checksum: u8 = buffer[0] + buffer[1] + buffer[2] + buffer[3];

    if checksum != buffer[4] {
      None
    } else {
      Some(Measurements {
        humidity: humidity,
        temperature: temperature,
      })
    }
  }

  fn wait_sync(&self) -> bool {
    if !self.wait_while(Low, 80) {
      false
    } else if !self.wait_while(High, 100) {
      false
    } else {
      true
    }
  }

  fn wait_while(&self, level: GpioLevel, timeout: usize) -> bool {
    for _ in 0..(timeout / 10) {
      self.timer.wait_us(10);
      if self.gpio.level() != level {
        return true;
      }
    }

    false
  }
}
