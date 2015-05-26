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

use hal::lpc17xx::peripheral_clock::PeripheralClock;
use hal::lpc17xx::peripheral_clock::PeripheralClock::PWM1Clock;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct PWMPeripheral {
    value: u32
}


/// LPC17xx PWM Register Definitions (User Manual: 24.6)
mod reg {
    use util::volatile_cell::VolatileCell;
    use core::ops::Drop;

    ioregs!(PWM1 = {
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
                1 => SYNCHRONOUS_RESET
            },
            // 2 => Reserved
            3 => pwm_enable {
                0 => DISABLED,
                1 => ENABLED
            }
        }
        /// Timer Counter. The 32-bit TC is incremented every PR+1
        /// cycles of PCLK.  The TC is controlled through the TCR.
        0x08 => reg32 tc {
        }
        /// Prescale Register. The TC is incremented every PR+1 cycles
        /// of PCLK.
        0x0C => reg32 pr {
        }
        /// Prescale Counter. The 32-bit PC is a counter which is
        /// incremented to the value stored in PR. When the value in
        /// PR is reached, the TC is incremented. The PC is observable
        /// and controllable through the bus interface.
        0x10 => reg32 pc {
            
        }
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

            15=> pwmmr5i,  //= if set, interrupt on pwmmr5
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
        /// in single-edge mode, and sets PWM<N + 1> if it’s in double-edge
        /// mode.
        0x18 => reg32 mr[4] {

        }
        /// Capture Control Register. The CCR controls which edges of
        /// the capture inputs are used to load the Capture Registers
        /// and whether or not an interrupt is generated when a
        /// capture takes place.
        0x28 => reg32 ccr {
        }
        /// Capture Registers (0-3). CR<N> is loaded with the value of the TC
        /// when there is an event on the CAPn.<N> input.
        0x30 => reg32 cr[4] {
        }
        /// Match Registers (4-6).  See `mr` registers.  Not sure why
        /// banks are fragmented.
        0x40 => reg32 mr2[3] {
        }
        /// PWM Control Register. Enables PWM outputs and selects PWM
        /// channel types as either single edge or double edge
        /// controlled.
        0x4c => reg32 pcr {
        }
        /// Load Enable Register. Enables use of new PWM match
        /// values.
        0x50 => reg32 ler {
        }
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

    extern {
        #[link_name="lpc17xx_iomem_PWM1"] pub static PWM1: PWM1;
    }

}
