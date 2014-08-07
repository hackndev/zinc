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

//! Interface to Nested Vector Interrupt Controller.
//  Link: http://infocenter.arm.com/help/topic/com.arm.doc.dui0552a/CIHIGCIF.html

#[path="../../lib/ioreg.rs"] mod ioreg;

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg_old!(ISERReg: u32, ISER0, ISER1, ISER2, ISER3,
        ISER4, ISER5, ISER6, ISER7)
  reg_rw!(ISERReg, u32, ISER0,   set_ISER0,      ISER0)
  reg_rw!(ISERReg, u32, ISER1,   set_ISER1,      ISER1)
  reg_rw!(ISERReg, u32, ISER2,   set_ISER2,      ISER2)
  reg_rw!(ISERReg, u32, ISER3,   set_ISER3,      ISER3)
  reg_rw!(ISERReg, u32, ISER4,   set_ISER4,      ISER4)
  reg_rw!(ISERReg, u32, ISER5,   set_ISER5,      ISER5)
  reg_rw!(ISERReg, u32, ISER6,   set_ISER6,      ISER6)
  reg_rw!(ISERReg, u32, ISER7,   set_ISER7,      ISER7)

  ioreg_old!(ICERReg: u32, ICER0, ICER1, ICER2, ICER3,
        ICER4, ICER5, ICER6, ICER7)
  reg_rw!(ICERReg, u32, ICER0,   set_ICER0,      ICER0)
  reg_rw!(ICERReg, u32, ICER1,   set_ICER1,      ICER1)
  reg_rw!(ICERReg, u32, ICER2,   set_ICER2,      ICER2)
  reg_rw!(ICERReg, u32, ICER3,   set_ICER3,      ICER3)
  reg_rw!(ICERReg, u32, ICER4,   set_ICER4,      ICER4)
  reg_rw!(ICERReg, u32, ICER5,   set_ICER5,      ICER5)
  reg_rw!(ICERReg, u32, ICER6,   set_ICER6,      ICER6)
  reg_rw!(ICERReg, u32, ICER7,   set_ICER7,      ICER7)

  ioreg_old!(ISPRReg: u32, ISPR0, ISPR1, ISPR2, ISPR3,
        ISPR4, ISPR5, ISPR6, ISPR7)
  reg_rw!(ISPRReg, u32, ISPR0,   set_ISPR0,      ISPR0)
  reg_rw!(ISPRReg, u32, ISPR1,   set_ISPR1,      ISPR1)
  reg_rw!(ISPRReg, u32, ISPR2,   set_ISPR2,      ISPR2)
  reg_rw!(ISPRReg, u32, ISPR3,   set_ISPR3,      ISPR3)
  reg_rw!(ISPRReg, u32, ISPR4,   set_ISPR4,      ISPR4)
  reg_rw!(ISPRReg, u32, ISPR5,   set_ISPR5,      ISPR5)
  reg_rw!(ISPRReg, u32, ISPR6,   set_ISPR6,      ISPR6)
  reg_rw!(ISPRReg, u32, ISPR7,   set_ISPR7,      ISPR7)

  ioreg_old!(ICPRReg: u32, ICPR0, ICPR1, ICPR2, ICPR3,
        ICPR4, ICPR5, ICPR6, ICPR7)
  reg_rw!(ICPRReg, u32, ICPR0,   set_ICPR0,      ICPR0)
  reg_rw!(ICPRReg, u32, ICPR1,   set_ICPR1,      ICPR1)
  reg_rw!(ICPRReg, u32, ICPR2,   set_ICPR2,      ICPR2)
  reg_rw!(ICPRReg, u32, ICPR3,   set_ICPR3,      ICPR3)
  reg_rw!(ICPRReg, u32, ICPR4,   set_ICPR4,      ICPR4)
  reg_rw!(ICPRReg, u32, ICPR5,   set_ICPR5,      ICPR5)
  reg_rw!(ICPRReg, u32, ICPR6,   set_ICPR6,      ICPR6)
  reg_rw!(ICPRReg, u32, ICPR7,   set_ICPR7,      ICPR7)

  ioreg_old!(IABRReg: u32, IABR0, IABR1, IABR2, IABR3,
        IABR4, IABR5, IABR6, IABR7)
  reg_rw!(IABRReg, u32, IABR0,   set_IABR0,      IABR0)
  reg_rw!(IABRReg, u32, IABR1,   set_IABR1,      IABR1)
  reg_rw!(IABRReg, u32, IABR2,   set_IABR2,      IABR2)
  reg_rw!(IABRReg, u32, IABR3,   set_IABR3,      IABR3)
  reg_rw!(IABRReg, u32, IABR4,   set_IABR4,      IABR4)
  reg_rw!(IABRReg, u32, IABR5,   set_IABR5,      IABR5)
  reg_rw!(IABRReg, u32, IABR6,   set_IABR6,      IABR6)
  reg_rw!(IABRReg, u32, IABR7,   set_IABR7,      IABR7)

  //TODO(bharrisau): Implement byte-level access for 240 Priority Registers
  ioreg_old!(IPRReg: u32, IPR0)
  reg_w!(IPRReg, u32, set_IPR0, IPR0)

  ioreg_old!(STIRReg: u32, STIR)
  reg_w!(STIRReg, u32, set_STIR, STIR)

  #[allow(dead_code)]
  extern {
    #[link_name="armmem_NVIC_ISER"] pub static ISER: ISERReg;
    #[link_name="armmem_NVIC_ICER"] pub static ICER: ICERReg;
    #[link_name="armmem_NVIC_ISPR"] pub static ISPR: ISPRReg;
    #[link_name="armmem_NVIC_ICPR"] pub static ICPR: ICPRReg;
    #[link_name="armmem_NVIC_IABR"] pub static IABR: IABRReg;
    #[link_name="armmem_NVIC_IPR"]  pub static IPR:  IPRReg;
    #[link_name="armmem_NVIC_STIR"] pub static STIR: STIRReg;
  }
}
