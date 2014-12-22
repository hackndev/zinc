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
//! 
//! System Control Block memory location is 0xE000_ED00.
//! System Control Block ACTLR memory location is 0xE000_E008;
//  Link: http://infocenter.arm.com/help/topic/com.arm.doc.dui0552a/CIHFDJCA.html

#[inline(always)]
fn get_reg() -> &'static reg::SCB {
  unsafe { &*(0xE000_ED00 as *mut reg::SCB) }
}

/// Returns the CPUID.
#[allow(dead_code)]
pub fn cpuid() -> reg::SCB_cpuid_Get {
  get_reg().cpuid.get()
}

/// Sets the pending state of the PendSV interrupt.
pub fn set_pendsv(val: bool) {
  if val {
    get_reg().icsr.set_pendsvset(true);
  } else {
    get_reg().icsr.set_pendsvclr(true);
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(SCB = {
    0x0       => reg32 cpuid { //! CPUID base register
      0..3    => revision,
      4..15   => partno,
      20..23  => variant,
      24..31  => implementer,
    }
    0x4       => reg32 icsr {  //! Interrupt control and state register
      0..8    => vectactive,
      11      => rettobase,
      12..20  => vectpending,
      22      => isrpending,
      23      => isrprempt,
      25      => pendstclr,
      26      => pendstset,
      27      => pendsvclr,
      28      => pendsvset,
      31      => nmipendset,
    }
    0x8       => reg32 vtor {  //! Vector table offset register
      7..31   => tbloff,
    }
    0xc       => reg32 aircr { //! Application interrupt and reset control register
      0       => vectreset,
      1       => vectclractive,
      2       => sysresetreq,
      8..10   => prigroup,
      15      => endianness,
      16..31  => vectkey,
    }
    0x10      => reg32 scr {   //! System control register
      1       => sleeponexit,
      2       => sleepdeep,
      4       => sevonpend,
    }
    0x14      => reg32 ccr {   //! Configuration and control register
      0       => nonbasethrdena,
      1       => usersetmpend,
      3       => unalign_trp,
      4       => div_0_trp,
      8       => bfhfnmign,
      9       => stkalign,
    }
    0x18      => reg32 shpr[3] { //! System handler priority register
      0..31   => pri[4],
    }
    0x24      => reg32 shcsr { //! System handler control and state register
      0       => memfaultact,
      1       => busfaultact,
      3       => usgfaultact,
      7       => svcallact,
      8       => monitoract,
      10      => pendsvact,
      11      => systickact,
      12      => usgfaultpended,
      13      => memfaultpended,
      14      => busfaultpended,
      15      => svfaultpended,
      16      => memfaultpendena,
      17      => busfaultena,
      18      => usgfaultena,
    }
    0x28      => reg32 cfsr {  //! Configurable fault status register
      0..7    => memmanage,
      8..15   => busfault,
      16..31  => usagefault,
    }
    0x2c      => reg32 hfsr {  //! HardFault status register
      1       => vecttbl,
      30      => forced,
      31      => debugevt,
    }
    0x30      => reg32 dfsr {  //! DebugFault status register
      0       => halted,
      1       => bkpt,
      2       => dwttrap,
      3       => vcatch,
      4       => external,
    }
    0x34      => reg32 mmfar { //! MemManage address register
      0..31   => address,
    }
    0x38      => reg32 bfar {  //! BusFault address register
      0..31   => address,
    }
    0x3c      => reg32 afsr {  //! Auxilary fault address register
      0..31   => afsr,
    }
    0x88      => reg32 cpacr { //! Coprocessor access control register
      0..23   => cp[24],
    }
  });
}
