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

#[path="../../lib/ioreg.rs"]
mod ioreg;

mod reg {
  use core::{volatile_load, volatile_store};

  #[allow(uppercase_variables)]
  pub struct RCC {
    CR: u32,
    PLLCFGR: u32,
    CFGR: u32,
    CIR: u32,
    AHB1RSTR: u32,
    AHB2RSTR: u32,
    AHB3RSTR: u32,
    _pad_0: u32,
    APB1RSTR: u32,
    APB2RSTR: u32,
    _pad_1: u32,
    _pad_2: u32,
    AHB1ENR: u32,
    AHB2ENR: u32,
    AHB3ENR: u32,
    _pad_3: u32,
    APB1ENR: u32,
    APB2ENR: u32,
    _pad_4: u32,
    _pad_5: u32,
    AHB1LPENR: u32,
    AHB2LPENR: u32,
    AHB3LPENR: u32,
    _pad_6: u32,
    APB1LPENR: u32,
    APB2LPENR: u32,
    _pad_7: u32,
    _pad_8: u32,
    BDCR: u32,
    CSR: u32,
    _pad_9: u32,
    _pad_10: u32,
    SSCGR: u32,
    PLLI2SCFGR: u32,
  }

  reg_rw!(RCC, CR,         set_CR,        CR)
  reg_rw!(RCC, PLLCFGR,    set_PLLCFGR,   PLLCFGR)
  reg_rw!(RCC, CFGR,       set_CFGR,      CFGR)
  reg_rw!(RCC, CIR,        set_CIR,       CIR)
  reg_rw!(RCC, AHB1RSTR,   set_AHB1RSTR,  AHB1RSTR)
  reg_rw!(RCC, AHB2RSTR,   set_AHB2RSTR,  AHB2RSTR)
  reg_rw!(RCC, AHB3RSTR,   set_AHB3RSTR,  AHB3RSTR)
  reg_rw!(RCC, APB1RSTR,   set_APB1RSTR,  APB1RSTR)
  reg_rw!(RCC, APB2RSTR,   set_APB2RSTR,  APB2RSTR)
  reg_rw!(RCC, AHB1ENR,    set_AHB1ENR,    AHB1ENR)
  reg_rw!(RCC, AHB2ENR,    set_AHB2ENR,    AHB2ENR)
  reg_rw!(RCC, AHB3ENR,    set_AHB3ENR,    AHB3ENR)
  reg_rw!(RCC, APB1ENR,    set_APB1ENR,    APB1ENR)
  reg_rw!(RCC, APB2ENR,    set_APB2ENR,    APB2ENR)
  reg_rw!(RCC, AHB1LPENR,  set_AHB1LPENR,  AHB1LPENR)
  reg_rw!(RCC, AHB2LPENR,  set_AHB2LPENR,  AHB2LPENR)
  reg_rw!(RCC, AHB3LPENR,  set_AHB3LPENR,  AHB3LPENR)
  reg_rw!(RCC, APB1LPENR,  set_APB1LPENR,  APB1LPENR)
  reg_rw!(RCC, APB2LPENR,  set_APB2LPENR,  APB2LPENR)
  reg_rw!(RCC, BDCR,       set_BDCR,       BDCR)
  reg_rw!(RCC, CSR,        set_CSR,        CSR)
  reg_rw!(RCC, SSCGR,      set_SSCGR,      SSCGR)
  reg_rw!(RCC, PLLI2SCFGR, set_PLLI2SCFGR, PLLI2SCFGR)

  pub static RCC : *mut RCC = 0x40023800 as *mut RCC;
}

#[allow(non_camel_case_types)]
pub enum Peripheral {
  GPIO_A = 1 << 0,
  GPIO_B = 1 << 1,
  GPIO_C = 1 << 2,
  GPIO_D = 1 << 3,
  GPIO_E = 1 << 4,
  GPIO_F = 1 << 5,
  GPIO_G = 1 << 6,
  GPIO_H = 1 << 7,
  GPIO_I = 1 << 8,
  CRC    = 1 << 12,
  BackupSRAM = 1 << 18,
  CCMDataRAM = 1 << 20,
  DMA_1  = 1 << 21,
  DMA_2  = 1 << 22,
  EthernetMAC = 1 << 25,
  EthernetTX  = 1 << 26,
  EthernetRX  = 1 << 27,
  EthernetPTP = 1 << 28,
  USB_OTG = 1 << 29,
  USB_OTG_HS_ULPI = 1 << 30,
}


/// TODO(farcaller): this deals with parts of AHB1 only for now
pub fn set_peripheral_clock(p: Peripheral, enabled: bool) {
  let bit: u32 = p as u32;
  let val: u32 = unsafe { (*reg::RCC).AHB1ENR() };
  let new_val: u32 = match enabled {
    true  => val | bit,
    false => val & !bit,
  };
  unsafe { (*reg::RCC).set_AHB1ENR(new_val) };
}
