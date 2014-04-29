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

//! Timer configuration for ST STM32F4.
//!
//! This code supports only TIM2 at the moment

use super::peripheral_clock;
use hal::timer;

#[path="../../lib/ioreg.rs"] mod ioreg;

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg!(TIM2To5: CR1, CR2, SMCR, DIER, SR, EGR, CCMR1, CCMR2, CCER, CNT,
                       PSC, ARR, _pad_0, CCR1, CCR2, CCR3, CCR4, _pad_1, DCR,
                       DMAR, OR)
  reg_rw!(TIM2To5, CR1,   set_CR1,   CR1)
  reg_rw!(TIM2To5, CR2,   set_CR2,   CR2)
  reg_rw!(TIM2To5, SMCR,  set_SMCR,  SMCR)
  reg_rw!(TIM2To5, DIER,  set_DIER,  DIER)
  reg_rw!(TIM2To5, SR,    set_SR,    SR)
  reg_w!( TIM2To5,        set_EGR,   EGR)
  reg_rw!(TIM2To5, CCMR1, set_CCMR1, CCMR1)
  reg_rw!(TIM2To5, CCMR2, set_CCMR2, CCMR2)
  reg_rw!(TIM2To5, CCER,  set_CCER,  CCER)
  reg_rw!(TIM2To5, CNT,   set_CNT,   CNT)
  reg_rw!(TIM2To5, PSC,   set_PSC,   PSC)
  reg_rw!(TIM2To5, ARR,   set_ARR,   ARR)
  reg_rw!(TIM2To5, CCR1,  set_CCR1,  CCR1)
  reg_rw!(TIM2To5, CCR2,  set_CCR2,  CCR2)
  reg_rw!(TIM2To5, CCR3,  set_CCR3,  CCR3)
  reg_rw!(TIM2To5, CCR4,  set_CCR4,  CCR4)
  reg_rw!(TIM2To5, DCR,   set_DCR,   DCR)
  reg_rw!(TIM2To5, DMAR,  set_DMAR,  DMAR)
  reg_rw!(TIM2To5, OR,    set_OR,    OR)

  extern {
    #[link_name="iomem_TIM2"] pub static TIM2: TIM2To5;
  }
}

/// Available timer peripherals.
pub enum TimerPeripheral {
  Timer2,
}

/// Configuration for timer.
pub struct TimerConf {
  /// Peripheral to use.
  pub timer: TimerPeripheral,
  /// Number of clock ticks to increment the counter.
  pub counter: u32,
}

pub struct Timer {
  reg: &'static reg::TIM2To5,
}

impl TimerConf {
  /// Returns a platform-specific timer object that implements Timer trait.
  pub fn setup(&self) -> Timer {
    let (clock, reg) = match self.timer {
      Timer2 => (peripheral_clock::TIM2Clock, &reg::TIM2),
    };

    clock.enable();

    reg.set_PSC(self.counter - 1);
    reg.set_CR1(1);
    reg.set_EGR(1);

    Timer {
      reg: reg,
    }
  }
}

impl timer::Timer for Timer {
  #[inline(always)]
  fn get_counter(&self) -> u32 {
    self.reg.CNT()
  }
}
