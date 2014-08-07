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
//  Link: http://infocenter.arm.com/help/topic/com.arm.doc.dui0552a/BIHJJABA.html

#[path="../../lib/ioreg.rs"] mod ioreg;

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg_old!(MPUReg: u32, TYPE, CTRL, RNR, RBAR, RASR)
  reg_r!( MPUReg, u32, TYPE,                     TYPE)
  reg_rw!(MPUReg, u32, CTRL,     set_CTRL,       CTRL)
  reg_rw!(MPUReg, u32, RNR,      set_RNR,        RNR)
  reg_rw!(MPUReg, u32, RBAR,     set_RBAR,       RBAR)
  reg_rw!(MPUReg, u32, RASR,     set_RASR,       RASR)

  #[allow(dead_code)]
  extern {
    #[link_name="armmem_MPU"] pub static MPU: MPUReg;
  }
}
