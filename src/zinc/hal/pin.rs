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

//! Common definitions for pin HAL.

pub use self::GpioDirection::*;
pub use self::GpioLevel::*;

/// GPIO direction.
#[derive(Copy)]
pub enum GpioDirection {
  /// Input mode.
  In,
  /// Output mode.
  Out,
}

/// Logic levels.
#[derive(PartialEq, Copy)]
pub enum GpioLevel {
  /// Logic low.
  Low,
  /// Logic high.
  High,
}

/// General Purpose I/O.
pub trait Gpio {
  /// Set to logic high.
  fn set_high(&self);

  /// Set to logic low.
  fn set_low(&self);

  /// Read current logic level.
  fn level(&self) -> GpioLevel;

  /// Set direction mode to `In` or `Out`,
  /// for reading or writing respectively.
  fn set_direction(&self, new_mode: GpioDirection);
}
