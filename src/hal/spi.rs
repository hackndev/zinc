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
SPI interface.

SPIConf is a MCU-specific struct.

As SPI performs read and write as one operation, special care should be taken if
`write()` and `read()` methods are used with several devices on one SPI
peripheral. The best way is to always use `transfer()`.
*/

/// SPI trait.
pub trait Spi {
  /// Writes a byte over SPI.
  ///
  /// It's implementation defined what happens if SPI is not configured to 8
  /// bits.
  fn write(&self, value: u8);

  /// Reads a byte from SPI.
  ///
  /// This function returns the last byte received (SPI sends and receives data
  /// at the same time).
  fn read(&self) -> u8;

  /// Performs an SPI transfer operation (writes one byte, returns the one byte
  /// read).
  fn transfer(&self, value: u8) -> u8 {
    self.write(value);
    self.read()
  }
}
