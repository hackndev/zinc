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

/*!
System Integration Module registers.
*/

use lib::volatile_cell::VolatileCell;

#[path="../../../lib/ioreg.rs"] mod ioreg;

ioreg!( SIMLPReg: SOPT1, SOPT1CFG)
reg_rw!(SIMLPReg, SOPT1,    set_SOPT1,    SOPT1)
reg_rw!(SIMLPReg, SOPT1CFG, set_SOPT1CFG, SOPT1CFG)

ioreg!( SIMReg: _pad_0, SOPT2, _pad_1, SOPT4, SOPT5, _pad_2, SOPT7, _pad_3,
        _pad_4, SDID, SCGC1, SCGC2, SCGC3, SCGC4, SCGC5, SCGC6,
        SCGC7, CLKDIV1, CLKDIV2, FCFG1, FCFG2, UIDH, UIDMH, UIDML,
        UIDL)
reg_rw!(SIMReg, SOPT2,      set_SOPT2,    SOPT2)
reg_rw!(SIMReg, SOPT2,      set_SOPT2,    SOPT2)
reg_rw!(SIMReg, SOPT4,      set_SOPT4,    SOPT4)
reg_rw!(SIMReg, SOPT5,      set_SOPT5,    SOPT5)
reg_rw!(SIMReg, SOPT7,      set_SOPT7,    SOPT7)
reg_rw!(SIMReg, SDID,       set_SDID,     SDID)
reg_rw!(SIMReg, SCGC1,      set_SCGC1,    SCGC1)
reg_rw!(SIMReg, SCGC2,      set_SCGC2,    SCGC2)
reg_rw!(SIMReg, SCGC3,      set_SCGC3,    SCGC3)
reg_rw!(SIMReg, SCGC4,      set_SCGC4,    SCGC4)
reg_rw!(SIMReg, SCGC5,      set_SCGC5,    SCGC5)
reg_rw!(SIMReg, SCGC6,      set_SCGC6,    SCGC6)
reg_rw!(SIMReg, SCGC7,      set_SCGC7,    SCGC7)
reg_rw!(SIMReg, CLKDIV1,    set_CLKDIV1,  CLKDIV1)
reg_rw!(SIMReg, CLKDIV2,    set_CLKDIV2,  CLKDIV2)
reg_rw!(SIMReg, FCFG1,      set_FCFG1,    FCFG1)
reg_rw!(SIMReg, FCFG2,      set_FCFG2,    FCFG2)
reg_rw!(SIMReg, UIDH,       set_UIDH,     UIDH)
reg_rw!(SIMReg, UIDMH,      set_UIDMH,    UIDMH)
reg_rw!(SIMReg, UIDML,      set_UIDML,    UIDML)
reg_rw!(SIMReg, UIDL,       set_UIDL,     UIDL)

extern {
  #[link_name="iomem_SIMLP"]  pub static SIMLP: SIMLPReg;
  #[link_name="iomem_SIM"]    pub static SIM:   SIMReg;
}
