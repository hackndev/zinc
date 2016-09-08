// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
// Adapted from stm32f4/timer.rs for stm32f7 by Dave Hylands <dhylands@gmail.com>
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

//! Timer configuration for ST STM32F7.
//!
//! This code supports only TIM2 at the moment.

use super::peripheral_clock;
use hal::timer;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

/// Available timer peripherals.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum TimerPeripheral {
  Timer2,
}

/// Structure describing a Timer.
#[derive(Clone, Copy)]
pub struct Timer {
  reg: &'static reg::TIM,
}

impl Timer {
  /// Create and start a Timer.
  pub fn new(peripheral: TimerPeripheral, counter: u32) -> Timer {
    use self::TimerPeripheral::*;
    let (clock, reg) = match peripheral {
      Timer2 => (peripheral_clock::PeripheralClock::TIM2Clock, &reg::TIM2),
    };

    clock.enable();

    reg.psc.set_prescaler(counter - 1);
    reg.cr1.set_counter_enabled(true);
    reg.egr.set_update_enabled(true);

    Timer {
      reg: reg,
    }
  }
}

impl timer::Timer for Timer {
  #[inline(always)]
  fn get_counter(&self) -> u32 {
    self.reg.cnt.counter()
  }
}

mod reg {
  use volatile_cell::VolatileCell;

  ioregs!(TIM = {
    0x00 => reg32 cr1 {         // Control Register 1           all
      11   => uif_remap_on : rw,
      9..8 => clock_divider : rw,
      7    => auto_reload_buffered : rw,
      6..5 => center_mode : rw,
      4    => dir_up : rw,
      3    => one_pulse_mode : rw,
      2    => update_request_source : rw,
      1    => uev_disabled : rw,
      0    => counter_enabled : rw,
    },
    0x04 => reg32 cr2 {         // Control Register 2           1-9, 10, 11, 13, 14
      23..20 => master_mode_selection_2 : rw,
      18..8  => output_idle_state : rw,
      7      => ti1_selection : rw,
      6..4   => master_mode_selection: rw,
      3      => cc_dma_update : rw,
      1      => cc_ctl_update : rw,
      0      => cc_preload_enabled : rw,
    },
    0x08 => reg32 smcr {        // Slave Mode Control Register  1-5, 8-14
      16     => slave_mode_selection_bit3 : rw,
      15     => ext_trigger_polarity_inverted : rw,
      14     => ext_clock_enabled : rw,
      13..12 => ext_trigger_prescaler : rw,
      11..8  => ext_trigger_filter : rw,
      7      => slave_trigger_sync : rw,
      6..4   => trigger_selection : rw,
      2..0   => slave_mode_selection : rw,
    },
    0x0c => reg32 dier {        // DMA/Interrupt Enable Reister all
      14 => trigger_dma_req_enabled : rw,
      13 => com_dma_req_enabled : rw,
      12 => cc4_dma_req_enabled : rw,
      11 => cc3_dma_req_enabled : rw,
      10 => cc2_dma_req_enabled : rw,
      9  => cc1_dma_req_enabled : rw,
      8  => update_dma_req_enabled : rw,
      7  => break_irq_enabled : rw,
      6  => trigger_irq_enabled : rw,
      5  => com_irq_enabled : rw,
      4  => cc4_irq_enabled : rw,
      3  => cc3_irq_enabled : rw,
      2  => cc2_irq_enabled : rw,
      1  => cc1_irq_enabled : rw,
      0  => update_irq_enabled : rw,
    },
    0x10 => reg32 sr {          // Status Register              all
      17 => cc6_irq_flag : rw,
      16 => cc5_irq_flag : rw,
      12 => cc4_overcapture_flag : rw,
      11 => cc3_overcapture_flag : rw,
      10 => cc2_overcapture_flag : rw,
      9  => cc1_overcapture_flag : rw,
      8  => break2_irq_flag : rw,
      7  => break_irq_flag : rw,
      6  => trigger_irq_flag : rw,
      5  => com_irq_flag : rw,
      4  => cc4_irq_flag : rw,
      3  => cc3_irq_flag : rw,
      2  => cc2_irq_flag : rw,
      1  => cc1_irq_flag : rw,
      0  => update_irq_flag : rw,
    },
    0x14 => reg32 egr {         // Event Generation Register    all
      8 => break2_enabled : rw,
      7 => break_enabled : rw,
      6 => trigger_enabled : rw,
      5 => cc_update_enabled : rw,
      4 => cc4_enabled : rw,
      3 => cc3_enabled : rw,
      2 => cc2_enabled : rw,
      1 => cc1_enabled : rw,
      0 => update_enabled : rw,
    },

    // The ccmr1 register is really a union of 2 other registers, but I haven't
    // figured out how to declare it. Delcaring 2 fields at the same offset
    // causes all of the following fields to get shifted.
    0x18 => reg32 ccmr1 {       // Compare Mode Register 1      1-5, 8-14
        31..0 => val : rw
    },
//    0x18 => reg32 ccmr1_oc {    // Compare Mode Register 1      1-5, 8-14
//      24     => oc2_mode_bit3 : rw,
//      16     => oc1_mode_bit3 : rw,
//      15     => oc2_clear_enable : rw,
//      14..12 => oc2_mode : rw,
//      11     => oc2_preload_enable : rw,
//      10     => oc2_fast_enable : rw,
//      9..8   => cc2_selection : rw,
//      7      => oc1_clear_enable : rw,
//      6..4   => oc1_mode : rw,
//      3      => oc1_preload_enable : rw,
//      2      => oc1_fast_enable : rw,
//      1..0   => cc1_selection : rw,
//    },
// Declaring a second register which has the same address as the previous one
// causes all of the subsequent ones to be shifted to an incorrect offset.
// 
//    0x18 => reg32 ccmr1_ic {    // Capture Mode Register 1      1-5, 8-14
//      15..12 => ic2_filter : rw,
//      11..10 => ic2_prescaler : rw,
//      9..8   => ic2_selection : rw,
//      7..4   => ic1_filter : rw,
//      3..2   => ic1_prescaler : rw,
//      1..0   => ic1_selection : rw,
//    },

    0x1c => reg32 ccmr2 {       // Compare Mode Register 2      1-5, 8-14
        31..0 => val : rw
    },
//    0x1c => reg32 ccmr2_oc {    // Compare Mode Register 2      1-5, 8-14
//      24     => oc4_mode_bit3 : rw,
//      16     => oc3_mode_bit3 : rw,
//      15     => oc4_clear_enable : rw,
//      14..12 => oc4_mode : rw,
//      11     => oc4_preload_enable : rw,
//      10     => oc4_fast_enable : rw,
//      9..8   => cc4_selection : rw,
//      7      => oc3_clear_enable : rw,
//      6..4   => oc3_mode : rw,
//      3      => oc3_preload_enable : rw,
//      2      => oc3_fast_enable : rw,
//      1..0   => cc3_selection : rw,
//    },
//    0x1c => reg32 ccmr2_ic {    // Capture Mode Register 2      1-5, 8-14
//      15..12 => ic4_filter : rw,
//      11..10 => ic4_prescaler : rw,
//      9..8   => ic4_selection : rw,
//      7..4   => ic3_filter : rw,
//      3..2   => ic3_prescaler : rw,
//      1..0   => ic3_selection : rw,
//    },
    0x20 => reg32 ccer {        // Capture/Compare Enable       1-5, 8-14
      21 => cc6_active_low : rw,
      20 => cc6_enabled : rw,
      17 => cc5_active_low : rw,
      16 => cc5_enabled : rw,
      15 => cc4n_active_low : rw,
      13 => cc4_active_low : rw,
      12 => cc4_enabled : rw,
      11 => cc3n_active_low : rw,
      10 => cc3n_enabled : rw,
      9  => cc3_active_low : rw,
      8  => cc3_enabled : rw,
      7  => cc2n_active_low : rw,
      6  => cc2n_enabled : rw,
      5  => cc2_active_low : rw,
      4  => cc2_enabled : rw,
      3  => cc1n_active_low : rw,
      2  => cc1n_enabled : rw,
      1  => cc1_active_low : rw,
      0  => cc1_enabled : rw,
    },
    0x24 => reg32 cnt {         // Counter Register             all
      31..0 => counter : rw,
    },
    0x28 => reg32 psc {         // Prescaler Register           all
      31..0 => prescaler : rw,
    },
    0x2c => reg32 arr {         // Auto-Reload Register         all
      31..0 => auto_reload : rw,
    },
    0x30 => reg32 rcr {         // Repetition Counter Register  1, 8
      31..0 => repetition_counter : rw,
    },
    0x34 => reg32 ccr1 {        // Capture/Compare Register 1   1-5, 8-14
      31..0 => value : rw,
    },
    0x38 => reg32 ccr2 {        // Capture/Compare Register 2   1-5, 8-14
      31..0 => value : rw,
    },
    0x3c => reg32 ccr3 {        // Capture/Compare Register 3   1-5, 8
      31..0 => value : rw,
    },
    0x40 => reg32 ccr4 {        // Capture/Compare Register 4   1-5, 8
      31..0 => value : rw,
    },
    0x44 => reg32 bdtr {        // Break/Dead-Time Register     1, 8
      25     => break2_active_high : rw,
      24     => break2_enabled : rw,
      23..20 => break2_filter : rw,
      19..16 => break_filter : rw,
      15     => main_output_enabled : rw,
      14     => auto_output_enabled : rw,
      13     => break_active_high : rw,
      12     => break_enabled : rw,
      11     => off_state_run_selection : rw,
      10     => off_state_idle_selection : rw,
      9..8   => lock : rw,
      7..0   => dead_time : rw,
    },
    0x48 => reg32 dcr {         // DMA Control Register         1-5, 8
      12..8 => dma_burst_len : rw,
      4..0  => dma_base_addr : rw,
    },
    0x4c => reg32 dmar {        // DMA Address Register         1-5, 8
      15..0 => dma_burst_data : rw,
    },
    0x50 => reg32 optr {        // Option Register              2-3, 5, 11
      11..10 => tim2_internal_trigger1_remap : rw,
      7..6   => tim5_timer_input4_remap : rw,
      1..0   => tim3_input_capture1_remap : rw, // Also tim11_input_capture1_remap
    },

    0x54 => reg32 ccmr3 {
      31..0 => val : rw,
    },
//    0x54 => reg32 ccmr3_oc {    // Compare Mode Register 3      1, 8
//      24     => oc6_mode_bit3 : rw,
//      16     => oc5_mode_bit3 : rw,
//      15     => oc6_clear_enable : rw,
//      14..12 => oc6_mode : rw,
//      11     => oc6_preload_enable : rw,
//      10     => oc6_fast_enable : rw,
//      9..8   => cc6_selection : rw,
//      7      => oc5_clear_enable : rw,
//      6..4   => oc5_mode : rw,
//      3      => oc5_preload_enable : rw,
//      2      => oc5_fast_enable : rw,
//      1..0   => cc5_selection : rw,
//    },
    0x58 => reg32 ccr5 {        // Capture/Compare Register 5   1, 8
      31..0 => value : rw,
    },
    0x5c => reg32 ccr6 {        // Capture/Compare Register 6   1, 8
      31..0 => value : rw,
    },
  });

  extern {
    #[link_name="stm32f7_iomem_TIM2"] pub static TIM2: TIM;
  }
}
