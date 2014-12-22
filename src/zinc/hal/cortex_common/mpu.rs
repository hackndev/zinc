// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Harris <mail@bharr.is>
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

//! Interface to Memory Protection Unit.
//! 
//! MPU memory location is 0xE000_ED90.
//  Link: http://infocenter.arm.com/help/topic/com.arm.doc.dui0552a/BIHJJABA.html

// TODO(bharrisau): Remove dead_code when MPU is implemented.
#[allow(dead_code)]
#[inline(always)]
fn get_reg() -> &'static reg::MPU {
  unsafe { &*(0xE000_ED90 as *mut reg::MPU) }
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(MPU = {
    0x0        => reg32 mpu_type { //! MPU type register
      0        => separate: ro,
      8..15    => dregion: ro,
      16..23   => iregion: ro,
    }
    0x4        => reg32 ctrl {     //= MPU control register
      0        => enable,
      1        => hfnmiena,
      2        => privdefena,
    }
    0x8        => reg32 rnr {      //! Region number register
      0..7     => region,
    }
    0xc        => reg32 rbar {     //! Region base address register
      0..3     => region,
      4        => valid,
      5..31    => addr,
    }
    0x10       => reg32 rasr {     //! Region attribute and size register
      0        => enable,
      1..5     => size,
      8..15    => srd,
      16       => b,
      17       => c,
      18       => s,
      19..21   => tex,
      24..26   => ap,
      28       => xn,
    }
  });
}
