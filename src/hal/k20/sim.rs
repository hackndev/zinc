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

//! HAL for Kinetis SIM module.

#[path="../../lib/ioreg.rs"] mod ioreg;

/// Enable clock to a PORTx peripheral
#[allow(non_snake_case_functions)]
pub fn enable_PORT(num: uint) {
  reg::SIM.set_SCGC5(reg::SIM.SCGC5() | (1 << (num + 8)));
}

#[allow(dead_code)]
mod reg {
  use lib::volatile_cell::VolatileCell;

  #[allow(uppercase_variables)]
  struct SIM {
    SOPT1:    VolatileCell<u32>,
    SOPT1CFG: VolatileCell<u32>,
    _pad0:    [VolatileCell<u32>, ..(0x1004 - 0x8) / 4],
    SOPT2:    VolatileCell<u32>,
    _pad1:    VolatileCell<u32>,
    SOPT4:    VolatileCell<u32>,
    SOPT5:    VolatileCell<u32>,
    _pad2:    VolatileCell<u32>,
    SOPT7:    VolatileCell<u32>,
    _pad3:    [VolatileCell<u32>, ..(0x1024 - 0x101c) / 4],
    SDID:     VolatileCell<u32>,
    _pad4:    [VolatileCell<u32>, ..(0x1034 - 0x1028) / 4],
    SCGC4:    VolatileCell<u32>,
    SCGC5:    VolatileCell<u32>,
    SCGC6:    VolatileCell<u32>,
    SCGC7:    VolatileCell<u32>,
    CLKDIV1:  VolatileCell<u32>,
    CLKDIV2:  VolatileCell<u32>,
    FCFG1:    VolatileCell<u32>,
    FCFG2:    VolatileCell<u32>,
  }

  reg_rw!(SIM, u32, SOPT5,  set_SOPT5,  SOPT5)
  reg_rw!(SIM, u32, SCGC5,  set_SCGC5,  SCGC5)

  extern {
    #[link_name="k20_iomem_SIM"] pub static SIM: SIM;
  }
}
