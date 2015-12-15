// Zinc, the bare metal stack for rust.
// Copyright 2015 Paul Osborne <osbpau@gmail.com>
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

//! PWM Support for the NXP LPC17xx MCUs

use hal::lpc17xx::peripheral_clock::PeripheralClock::PWM1Clock;
use core::intrinsics::abort;

use self::PWMChannel::*;


#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

const PWM_CLOCK_DIVISOR: u8 = 4;

/// "Channels" correspond to MR1..6 (0 used for period)
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum PWMChannel {
  Channel0PeriodReserved = 0,  // reserved for period
  Channel1,
  Channel2,
  Channel3,
  Channel4,
  Channel5,
  Channel6,
}

impl PWMChannel {
  /// Set the match register for this channel to the provided value
  fn set_mr(&self, value: u32) {
    // note that for whatever reason, the match registers are fragmented
    // across two registers.  They aren't even located next to each other
    // in memory
    match *self {
      Channel0PeriodReserved => { reg::PWM1().mr[0].set_value(value); },
      Channel1 => { reg::PWM1().mr[1].set_value(value); },
      Channel2 => { reg::PWM1().mr[2].set_value(value); },
      Channel3 => { reg::PWM1().mr[3].set_value(value); },
      Channel4 => { reg::PWM1().mr2[0].set_value(value); },
      Channel5 => { reg::PWM1().mr2[1].set_value(value); },
      Channel6 => { reg::PWM1().mr2[2].set_value(value); },
    };
  }
}

/// calculate the number of PWM clock ticks for the given number of microseconds
fn pwm_us_to_ticks(us: u32) -> u32 {
  let pwm_clock_frequency_hz: u32 = PWM1Clock.frequency();
  let pwm_clock_frequency_mhz: u32 = pwm_clock_frequency_hz / 1_000_000;
  return pwm_clock_frequency_mhz * us;
}


#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct PWM {
  channel: PWMChannel,
  period_us: u32,
  pulsewidth_us: u32,
}

impl PWM {
  /// Create a new PWM Output on the provided channel
  ///
  /// 20ms is a common period for PWM signal (20_000 us)
  pub fn new(channel: PWMChannel, period_us: u32) -> PWM {
    PWM1Clock.enable();
    PWM1Clock.set_divisor(PWM_CLOCK_DIVISOR);
    reg::PWM1().pr.set_value(0);  // no prescaler

    // single PWM mode (reset TC on match 0 for Ch0)
    reg::PWM1().mcr.set_pwmmr0r(true);

    // enable PWM output on this channel
    match channel {
      Channel0PeriodReserved => { unsafe { abort() } },  // Channel0 reserved for internal use
      Channel1 => { reg::PWM1().pcr.set_pwmena1(true); },
      Channel2 => { reg::PWM1().pcr.set_pwmena2(true); },
      Channel3 => { reg::PWM1().pcr.set_pwmena3(true); },
      Channel4 => { reg::PWM1().pcr.set_pwmena4(true); },
      Channel5 => { reg::PWM1().pcr.set_pwmena5(true); },
      Channel6 => { reg::PWM1().pcr.set_pwmena6(true); },
    };

    let pwm = PWM {
      channel: channel,
      period_us: period_us,  // 20ms is pretty common
      pulsewidth_us: 0,
    };

    pwm.update_period();
    pwm.update_pulsewidth();
    pwm
  }

  /// Update the PWM Signal based on the current state
  fn update_period(&self) {
    // Put the counter into reset and disable the counter
    reg::PWM1().tcr
      .set_ctr_en(reg::PWM1_tcr_ctr_en::DISABLED)
      .set_ctr_reset(reg::PWM1_tcr_ctr_reset::RESET)
      .set_pwm_enable(reg::PWM1_tcr_pwm_enable::DISABLED);

    // setup match register to ticks per period on CH0
    Channel0PeriodReserved.set_mr(pwm_us_to_ticks(self.period_us));

    // TODO(posborne): recalculate other registers based on the new period?

    // set the channel latch to update CH0 and CHN
    reg::PWM1().ler.set_value(1 << Channel0PeriodReserved as u32);

    // enable counter and pwm; clear reset
    reg::PWM1().tcr
      .set_ctr_en(reg::PWM1_tcr_ctr_en::ENABLED)
      .set_ctr_reset(reg::PWM1_tcr_ctr_reset::CLEAR_RESET)
      .set_pwm_enable(reg::PWM1_tcr_pwm_enable::ENABLED);
  }

  fn update_pulsewidth(&self) {
    let mut pulsewidth_ticks: u32 = pwm_us_to_ticks(self.pulsewidth_us);
    if pulsewidth_ticks == reg::PWM1().mr[0].get().raw() {
      // avoid making it equal or there is a 1 cycle dropout
      pulsewidth_ticks = pulsewidth_ticks + 1;
    }
    self.channel.set_mr(pulsewidth_ticks);
    reg::PWM1().ler.set_value(1 << self.channel as u32);
  }

}

/// Implementation of Generic PWMOutput trait for LPC17xx
impl ::hal::pwm::PWMOutput for PWM {
  fn set_period_us(&mut self, period_us: u32) {
    self.period_us = period_us;
    self.update_period();
  }

  fn get_period_us(&self) -> u32 {
    self.period_us
  }

  fn set_pulsewidth_us(&mut self, pulsewidth_us: u32) {
    self.pulsewidth_us = pulsewidth_us;
    self.update_pulsewidth();
  }

  fn get_pulsewidth_us(&self) -> u32 {
    self.pulsewidth_us
  }
}


/// LPC17xx PWM Register Definitions (User Manual: 24.6)
mod reg {
  use volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(PWM1@0x40018000 = {
    /// Interrupt Register. The IR can be written to clear
    /// interrupts. The IR can be read to identify which of eight
    /// possible interrupt sources are pending.
    0x00 => reg32 ir {
      0  => irq_mr0,  //= Interrupt flag for PWM match channel 0.
      1  => irq_mr1,  //= Interrupt flag for PWM match channel 1.
      2  => irq_mr2,  //= Interrupt flag for PWM match channel 2.
      3  => irq_mr3,  //= Interrupt flag for PWM match channel 3.
      4  => irq_cap0, //= Interrupt flag for capture input 0
      5  => irq_cap1, //= Interrupt flag for capture input 1.
      // 7:6 Reserved
      8  => irq_mr4,  //= Interrupt flag for PWM match channel 4.
      9  => irq_mr5,  //= Interrupt flag for PWM match channel 5.
      10 => irq_mr6,  //= Interrupt flag for PWM match channel 6.
    }
    /// Timer Control Register. The TCR is used to control the
    /// Timer Counter functions. The Timer Counter can be disabled
    /// or reset through the TCR.
    0x04 => reg32 tcr {
      0 => ctr_en {
         0 => DISABLED,
         1 => ENABLED
      },
      1 => ctr_reset {
        0 => CLEAR_RESET,
        1 => RESET
      },
      // 2 => Reserved
      3 => pwm_enable {
        0 => DISABLED,
        1 => ENABLED
      }
    }
    /// Timer Counter. The 32-bit TC is incremented every PR+1
    /// cycles of PCLK.  The TC is controlled through the TCR.
    0x08 => reg32 tc { 31..0 => value }
    /// Prescale Register. The TC is incremented every PR+1 cycles
    /// of PCLK.
    0x0C => reg32 pr { 31..0 => value }
    /// Prescale Counter. The 32-bit PC is a counter which is
    /// incremented to the value stored in PR. When the value in
    /// PR is reached, the TC is incremented. The PC is observable
    /// and controllable through the bus interface.
    0x10 => reg32 pc { 31..0 => value }
    /// Match Control Register. The MCR is used to control if an
    /// interrupt is generated and if the TC is reset when a Match
    /// occurs.
    0x14 => reg32 mcr {
      0 => pwmmr0i,  //= if set, interrupt on pwmmr0
      1 => pwmmr0r,  //= if set, reset on pwmmr0
      2 => pwmmr0s,  //= if set, stop on pwmmr0

      3 => pwmmr1i,  //= if set, interrupt on pwmmr1
      4 => pwmmr1r,  //= if set, reset on pwmmr1
      5 => pwmmr1s,  //= if set, stop on pwmmr1

      6 => pwmmr2i,  //= if set, interrupt on pwmmr2
      7 => pwmmr2r,  //= if set, reset on pwmmr2
      8 => pwmmr2s,  //= if set, stop on pwmmr2

      9 => pwmmr3i,  //= if set, interrupt on pwmmr3
      10 => pwmmr3r,  //= if set, reset on pwmmr3
      11 => pwmmr3s,  //= if set, stop on pwmmr3

      12 => pwmmr4i,  //= if set, interrupt on pwmmr4
      13 => pwmmr4r,  //= if set, reset on pwmmr4
      14 => pwmmr4s,  //= if set, stop on pwmmr4

      15 => pwmmr5i,  //= if set, interrupt on pwmmr5
      16 => pwmmr5r,  //= if set, reset on pwmmr5
      17 => pwmmr5s,  //= if set, stop on pwmmr5

      18 => pwmmr6i,  //= if set, interrupt on pwmmr6
      19 => pwmmr6r,  //= if set, reset on pwmmr6
      20 => pwmmr6s,  //= if set, stop on pwmmr6
    }
    /// Match Registers (0-3).  MR<N> can be enabled in the MCR to reset
    /// the TC, stop both the TC and PC, and/or generate an
    /// interrupt when it matches the TC.  In addition, a match
    /// between this value and the TC sets any PWM output that is
    /// in single-edge mode, and sets PWM<N + 1> if it is in double-edge mode
    0x18 => reg32 mr[4] { 31..0 => value }
    /// Capture Control Register. The CCR controls which edges of
    /// the capture inputs are used to load the Capture Registers
    /// and whether or not an interrupt is generated when a
    /// capture takes place.
    0x28 => reg32 ccr { 31..0 => value }
    /// Capture Registers (0-3). CR<N> is loaded with the value of the TC
    /// when there is an event on the CAPn.<N> input.
    0x30 => reg32 cr[4] { 31..0 => value }
    /// Match Registers (4-6).  See `mr` registers.  Not sure why
    /// banks are fragmented.
    0x40 => reg32 mr2[3] { 31..0 => value }
    /// PWM Control Register. Enables PWM outputs and selects PWM
    /// channel types as either single edge or double edge
    /// controlled.
    0x4c => reg32 pcr {
      9  => pwmena1,
      10 => pwmena2,
      11 => pwmena3,
      12 => pwmena4,
      13 => pwmena5,
      14 => pwmena6,
    }
    /// Load Enable Register. Enables use of new PWM match
    /// values.
    0x50 => reg32 ler { 31..0 => value }
    /// Count Control Register. The CTCR selects between Timer and
    /// Counter mode, and in Counter mode selects the signal and
    /// edge(s) for counting.
    0x70 => reg32 ctcr {
      0..2 => mode {
        0 => TIMER_MODE,
        1 => COUNTER_MODE_RISING,
        2 => COUNTER_MODE_FALLING,
        3 => COUNTER_MODE_BOTH
      }
    }
  });

}
