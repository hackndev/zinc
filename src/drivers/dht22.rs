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

use std::option::{Option, Some, None};
use std::iter::{Iterator, range};

use hal::gpio::{GPIO, GPIOConf, Low, High, In, Out, Level};
use hal::timer::Timer;

/// Basic DHT22 driver ported over from arduino example.
///
/// TODO(farcaller): this driver doesn't conform to zinc's xxxConf layout.
pub struct DHT22<'a, T> {
  gpio: GPIO<'a>,
  timer: &'a T,
}

pub struct Measurements {
  pub humidity: f32,
  pub temperature: f32,
}

impl<'a, T: Timer> DHT22<'a, T> {
  /// Creates a new DHT22 driver based on I/O GPIO and a timer with 10us resolution.
  pub fn new(gpio: &'a GPIOConf, timer: &'a T) -> DHT22<'a, T> {
    DHT22 {
      gpio: gpio.setup(),
      timer: timer,
    }
  }

  /// Returns previous sensor measurements or None if synchronization failed.
  pub fn read(&self) -> Option<Measurements> {
    let buffer: &mut [u8, ..5] = &mut [0, ..5];
    let mut idx: uint = 0;
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

    for _ in range(0, 40) {
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

    let humidity: f32 = ((buffer[0] as u16 << 8) | buffer[1] as u16) as f32 * 0.1;
    let temperature: f32 = if buffer[2] & 0x80 != 0 {
      -0.1 * (((buffer[2] as u16 & 0x7F) << 8) | buffer[3] as u16) as f32
    } else {
      0.1 * ((buffer[2] as u16 << 8) | buffer[3] as u16) as f32
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

  fn wait_while(&self, level: Level, timeout: uint) -> bool {
    for _ in range(0, timeout / 10) {
      self.timer.wait_us(10);
      if self.gpio.level() != level {
        return true;
      }
    }

    false
  }
}
