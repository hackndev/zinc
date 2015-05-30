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

//! HAL for STM32F4 GPIO peripheral.

use hal::stm32f4::pin;
use hal::pin::{GpioDirection};

#[path="../../util/ioreg.rs"]
mod ioreg;

/// Configuration for a GPIO.
#[derive(Clone, Copy)]
pub struct GPIOConf {
  /// Pin configuration for this GPIO.
  pub pin: pin::PinConf,

  /// Direction for GPIO, either `In` or `Out`.
  pub direction: GpioDirection,
}

impl GPIOConf {
  /// Returns a GPIO object (actually -- self), that can be used to toggle or
  /// read GPIO value.
  pub fn setup<'a>(&'a self) -> &'a pin::PinConf {
    self.pin.setup();

    &self.pin
  }
}
