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

use super::peripheral_clock;
use hal::timer;

#[path="../../lib/ioreg.rs"] mod ioreg;
#[path="../../lib/wait_for.rs"] mod wait_for;

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg_cell!(TIMER: IR, TCR, TC, PR, PC, MCR, MR0, MR1, MR2, MR3, CCR, CR0, CR1, EMR, CTCR)
  reg_cell_rw!(TIMER, IR,  set_IR,  IR)
  reg_cell_rw!(TIMER, TCR, set_TCR, TCR)
  reg_cell_rw!(TIMER, TC, set_TC, TC)
  reg_cell_rw!(TIMER, PR, set_PR, PR)
  reg_cell_rw!(TIMER, PC, set_PC, PC)
  reg_cell_rw!(TIMER, MCR, set_MCR, MCR)
  reg_cell_rw!(TIMER, MR0, set_MR0, MR0)
  reg_cell_rw!(TIMER, MR1, set_MR1, MR1)
  reg_cell_rw!(TIMER, MR2, set_MR2, MR2)
  reg_cell_rw!(TIMER, MR3, set_MR3, MR3)
  reg_cell_rw!(TIMER, CCR, set_CCR, CCR)
  reg_cell_rw!(TIMER, CR0, set_CR0, CR0)
  reg_cell_rw!(TIMER, CR1, set_CR1, CR1)
  reg_cell_rw!(TIMER, EMR, set_EMR, EMR)
  reg_cell_rw!(TIMER, CTCR, set_CTCR, CTCR)

  extern {
    #[link_name="iomem_TIMER0"] pub static TIMER0: TIMER;
    #[link_name="iomem_TIMER1"] pub static TIMER1: TIMER;
    #[link_name="iomem_TIMER2"] pub static TIMER2: TIMER;
    #[link_name="iomem_TIMER3"] pub static TIMER3: TIMER;
  }
}

/// Available timer peripherals.
pub enum TimerPeripheral {
  Timer0,
  Timer1,
  Timer2,
  Timer3,
}

/// Configuration for timer.
pub struct TimerConf {
  /// Peripheral to use.
  pub timer: TimerPeripheral,
  /// Number of clock ticks to increment the counter.
  pub counter: u32,
  /// Clock divisor.
  pub divisor: u8,
}

pub struct Timer {
  reg: &'static reg::TIMER,
}

impl TimerConf {
  pub fn setup(&self) -> Timer {
    let (clock, reg) = match self.timer {
      Timer0 => (peripheral_clock::TIM0Clock, &reg::TIMER0),
      Timer1 => (peripheral_clock::TIM1Clock, &reg::TIMER1),
      Timer2 => (peripheral_clock::TIM2Clock, &reg::TIMER2),
      Timer3 => (peripheral_clock::TIM3Clock, &reg::TIMER3),
    };

    clock.enable();
    clock.set_divisor(self.divisor);

    reg.set_CTCR(0);
    reg.set_TCR(2);
    reg.set_PR(self.counter - 1);
    reg.set_TCR(1);

    Timer {
      reg: reg,
    }
  }
}

impl timer::Timer for Timer {
  #[inline(always)]
  fn wait_us(&self, us: u32) {
    let start = self.reg.TC();
    wait_for!((self.reg.TC() - start) >= us);
  }
}
