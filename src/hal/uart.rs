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

/*!
UART interface.

UARTConf is a MCU-specific struct.

UART objects implement CharIO trait to perform actual data transmission.
*/

/// UART parity mode.
#[derive(Clone, Copy)]
pub enum Parity {
  /// Partity disabled.
  Disabled,
  /// Partity bit added to make number of 1s odd.
  Odd,
  /// Partity bit added to make number of 1s even.
  Even,
  /// Partity bit forced to 1.
  Forced1,
  /// Partity bit forced to 0.
  Forced0,
}
