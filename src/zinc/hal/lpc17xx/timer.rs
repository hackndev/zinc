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

/*!
Timer configuration.

This code supports all four primary timers of the MCU.
*/

use hal::timer;

use self::TimerPeripheral::*;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

/// Available timer peripherals.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum TimerPeripheral {
  Timer0,
  Timer1,
  Timer2,
  Timer3,
}

/// Configuration for timer.
#[derive(Copy)]
pub struct TimerConf {
  /// Peripheral to use.
  pub timer: TimerPeripheral,
  /// Number of clock ticks to increment the counter.
  pub counter: u32,
  /// Clock divisor.
  pub divisor: u8,
}

/// Struct describing a timer instance.
#[derive(Copy)]
pub struct Timer {
  reg: &'static reg::TIMER,
}

impl Timer {
  /// Create an start a timer.
  pub fn new(peripheral: TimerPeripheral, counter: u32, divisor: u8) -> Timer {
    use hal::lpc17xx::peripheral_clock::PeripheralClock as Clock;
    let (clock, reg) = match peripheral {
      Timer0 => (Clock::TIM0Clock, &reg::TIMER0),
      Timer1 => (Clock::TIM1Clock, &reg::TIMER1),
      Timer2 => (Clock::TIM2Clock, &reg::TIMER2),
      Timer3 => (Clock::TIM3Clock, &reg::TIMER3),
    };

    clock.enable();
    clock.set_divisor(divisor);

    reg.set_CTCR(0);
    reg.set_TCR(2);
    reg.set_PR(counter - 1);
    reg.set_TCR(1);

    Timer {
      reg: reg,
    }
  }
}

impl timer::Timer for Timer {
  #[inline(always)]
  fn get_counter(&self) -> u32 {
    self.reg.TC()
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(TIMER: u32, IR, TCR, TC, PR, PC, MCR, MR0, MR1, MR2, MR3, CCR, CR0, CR1, EMR, CTCR);
  reg_rw!(TIMER, u32, IR,  set_IR,  IR);
  reg_rw!(TIMER, u32, TCR, set_TCR, TCR);
  reg_rw!(TIMER, u32, TC, set_TC, TC);
  reg_rw!(TIMER, u32, PR, set_PR, PR);
  reg_rw!(TIMER, u32, PC, set_PC, PC);
  reg_rw!(TIMER, u32, MCR, set_MCR, MCR);
  reg_rw!(TIMER, u32, MR0, set_MR0, MR0);
  reg_rw!(TIMER, u32, MR1, set_MR1, MR1);
  reg_rw!(TIMER, u32, MR2, set_MR2, MR2);
  reg_rw!(TIMER, u32, MR3, set_MR3, MR3);
  reg_rw!(TIMER, u32, CCR, set_CCR, CCR);
  reg_rw!(TIMER, u32, CR0, set_CR0, CR0);
  reg_rw!(TIMER, u32, CR1, set_CR1, CR1);
  reg_rw!(TIMER, u32, EMR, set_EMR, EMR);
  reg_rw!(TIMER, u32, CTCR, set_CTCR, CTCR);

  extern {
    #[link_name="lpc17xx_iomem_TIMER0"] pub static TIMER0: TIMER;
    #[link_name="lpc17xx_iomem_TIMER1"] pub static TIMER1: TIMER;
    #[link_name="lpc17xx_iomem_TIMER2"] pub static TIMER2: TIMER;
    #[link_name="lpc17xx_iomem_TIMER3"] pub static TIMER3: TIMER;
  }
}
