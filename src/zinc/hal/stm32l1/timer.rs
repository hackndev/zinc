// Zinc, the bare metal stack for rust.
// Copyright 2014 Dzmitry "kvark" Malyshau <kvarkus@gmail.com>
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

//! Timer configuration for ST STM32L1.
//!
//! This code supports only TIM2 at the moment.

use hal::timer;

#[path="../../util/ioreg.rs"] mod ioreg;

/// Available timer peripherals.
#[allow(missing_doc)]
pub enum TimerPeripheral {
  Timer2,
}

/// Structure describing a Timer.
pub struct Timer {
  reg: &'static reg::TIMER,
}

impl Timer {
  /// Create and start a Timer.
  pub fn new(peripheral: TimerPeripheral, counter: u32) -> Timer {
    use super::peripheral_clock as pc;
    let (clock, reg) = match peripheral {
      Timer2 => (pc::ClockApb1(pc::Tim2), &reg::TIM2),
    };

    clock.enable();

    reg.cr1.set_control(1);
    reg.psc.set_prescaler(counter as u16 - 1);
    reg.egr.set_generate(1);

    Timer {
      reg: reg,
    }
  }
}

impl timer::Timer for Timer {
  #[inline(always)]
  fn get_counter(&self) -> u32 {
    self.reg.cnt.counter() as u32
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(TIMER = {
    0x00 => reg16 cr1 {      // control 1
      15..0 => control : rw,
    },
    0x04 => reg16 cr2 {      // control 2
      15..0 => control : rw,
    },
    0x08 => reg16 smcr {     // slave mode control
      15..0 => slave_control : rw,
    },
    0x0A => reg16 dier {     // DMA/interrupt enable
      15..0 => enable : rw,
    },
    0x10 => reg16 sr {       // status
      15..0 => status : rw,
    },
    0x14 => reg16 egr {      // event generation
      15..0 => generate : wo,
    },
    0x18 => reg16 ccmr1 {    // capture/compare mode 1
      15..0 => mode : rw,
    },
    0x1C => reg16 ccmr2 {    // capture/compare mode 2
      15..0 => mode : rw,
    },
    0x20 => reg16 ccer {     // capture/compare enable
      15..0 => enable : rw,
    },
    0x24 => reg16 cnt {      // counter
      15..0 => counter : rw,
    },
    0x28 => reg16 psc {      // prescaler
      15..0 => prescaler : rw,
    },
    0x2C => reg32 arr {      // auto-reload
      31..0 => reload : rw,
    },
    0x34 => reg32 ccr1 {     // capture/compare 1
      31..0 => cc : rw,
    },
    0x38 => reg32 ccr2 {     // capture/compare 2
      31..0 => cc : rw,
    },
    0x3C => reg32 ccr3 {     // capture/compare 3
      31..0 => cc : rw,
    },
    0x40 => reg32 ccr4 {     // capture/compare 4
      31..0 => cc : rw,
    },
    0x48 => reg16 dcr {      // DMA control
      15..0 => control : rw,
    },
    0x4C => reg16 dmap {     // DMA address for full transfer
      15..0 => address : rw,
    },
    0x50 => reg16 or {       // option
      15..0 => option : rw,
    },
  })

  extern {
    #[link_name="stm32l1_iomem_TIM2"] pub static TIM2: TIMER;
  }
}
