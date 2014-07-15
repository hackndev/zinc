// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

pub struct PeripheralInstance {
  name: String,
  register_type: Path,
  offset: u32,
}

/// A bit-band window
pub struct BitBand {
  /// Offset of bitband window
  window_offset: u32,
  /// Offset of shadowed region
  shadowed_offset: u32,
  /// Length of shadowed region in bytes
  shadowed_length: u32,
}

impl BitBand {
  /// Address of a given bit in a bit-band window
  pub fn bit_address(&self, address: u32, bit: u32) -> Option<u32> {
    let shadowed_end = self.shadowed_offset + self.shadowed_length;
    if address < self.shadowed_offset || address > shadowed_end {
      return None;
    } else {
      return Some(self.window_offset + (address - self.shadowed_offset)*8 + bit);
    }
  }
}

pub struct MemoryMap {
  bitbands: Vec<BitBand>,
  peripherals: Vec<PeripheralInstance>,
}
