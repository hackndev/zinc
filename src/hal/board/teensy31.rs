// Zinc, the bare metal stack for rust.
// Copyright 2014 Dawid Ciężarkiewicz <dpc@ucore.info>
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

//! Common board routines for Teensy Board 3.1

use hal::k20::pin;
use hal;
use core::option::Some;

/// Get in-built LED port
pub fn open_led() -> pin::Pin {
  pin::Pin::new(pin::PortC, 5, pin::GPIO, Some(hal::pin::Out))
}
