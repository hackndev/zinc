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

//! Interface to System Control Block.
//  Link: http://infocenter.arm.com/help/topic/com.arm.doc.dui0552a/CIHFDJCA.html

#[path="../../lib/ioreg.rs"] mod ioreg;

/// Returns the CPUID.
#[allow(dead_code)]
pub fn cpuid() -> u32 {
  reg::SCB.CPUID()
}

/// Sets the pending state of the PendSV interrupt.
pub fn set_pendsv(val: bool) {
  if val {
    reg::SCB.set_ICSR(1 << 28);
  } else {
    reg::SCB.set_ICSR(1 << 27);
  }
}

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(SCBACTLRReg: u32, ACTLR)
  reg_rw!(SCBACTLRReg, u32, ACTLR, set_ACTLR, ACTLR)

  ioreg!(SCBReg: u32, CPUID, ICSR, VTOR, AIRCR, SCR, CCR, SHPR1, SHPR2,
         SHPR3, SHCRS, CFSR, HFSR, _pad_0, MMAR, BFAR, AFSR)
  reg_r!( SCBReg, u32, CPUID,                    CPUID)
  reg_rw!(SCBReg, u32, ICSR,     set_ICSR,       ICSR)
  reg_rw!(SCBReg, u32, VTOR,     set_VTOR,       VTOR)
  reg_rw!(SCBReg, u32, AIRCR,    set_AIRCR,      AIRCR)
  reg_rw!(SCBReg, u32, SCR,      set_SCR,        SCR)
  reg_rw!(SCBReg, u32, CCR,      set_CCR,        CCR)
  reg_rw!(SCBReg, u32, SHPR1,    set_SHPR1,      SHPR1)
  reg_rw!(SCBReg, u32, SHPR2,    set_SHPR2,      SHPR2)
  reg_rw!(SCBReg, u32, SHPR3,    set_SHPR3,      SHPR3)
  reg_rw!(SCBReg, u32, SHCRS,    set_SHCRS,      SHCRS)
  reg_rw!(SCBReg, u32, CFSR,     set_CFSR,       CFSR)
  reg_rw!(SCBReg, u32, HFSR,     set_HFSR,       HFSR)
  reg_rw!(SCBReg, u32, MMAR,     set_MMAR,       MMAR)
  reg_rw!(SCBReg, u32, BFAR,     set_BFAR,       BFAR)
  reg_rw!(SCBReg, u32, AFSR,     set_AFSR,       AFSR)

  #[allow(dead_code)]
  extern {
    #[link_name="armmem_SCB"] pub static SCB: SCBReg;
    #[link_name="armmem_SCB_ACTLR"] pub static SCB_ACTLR: SCBACTLRReg;
  }
}
