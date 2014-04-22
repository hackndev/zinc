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

#[cfg(mcu_lpc17xx)] pub use hal::lpc17xx::ssp::SPIConf;

pub trait SPI {
  fn write(&self, value: u8);
  fn read(&self) -> u8;

  fn transfer(&self, value: u8) -> u8 {
    self.write(value);
    self.read()
  }
}
