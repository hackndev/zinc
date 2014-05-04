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

use std::option::{Option, Some};

extern {
  fn isr_hang();
}

static ISRCount: uint = 35;

#[link_section=".isr_vector_nvic"]
#[no_mangle]
pub static NVICVectors: [Option<extern unsafe fn()>, ..ISRCount] = [
  // s.a. lpc17xx user manual, table 50 (chapter 6.3)
  Some(isr_hang),         // isr_wdt
  Some(isr_hang),         // isr_timer_0
  Some(isr_hang),         // isr_timer_1
  Some(isr_hang),         // isr_timer_2
  Some(isr_hang),         // isr_timer_3
  Some(isr_hang),         // isr_uart_0
  Some(isr_hang),         // isr_uart_1
  Some(isr_hang),         // isr_uart_2
  Some(isr_hang),         // isr_uart_3
  Some(isr_hang),         // isr_pwm_1
  Some(isr_hang),         // isr_i2c_0
  Some(isr_hang),         // isr_i2c_1
  Some(isr_hang),         // isr_i2c_2
  Some(isr_hang),         // isr_spi
  Some(isr_hang),         // isr_ssp_0
  Some(isr_hang),         // isr_ssp_1
  Some(isr_hang),         // isr_pll_0
  Some(isr_hang),         // isr_rtc
  Some(isr_hang),         // isr_eint_0
  Some(isr_hang),         // isr_eint_1
  Some(isr_hang),         // isr_eint_2
  Some(isr_hang),         // isr_eint_3
  Some(isr_hang),         // isr_adc
  Some(isr_hang),         // isr_bod
  Some(isr_hang),         // isr_usb
  Some(isr_hang),         // isr_can
  Some(isr_hang),         // isr_dma
  Some(isr_hang),         // isr_i2s
  Some(isr_hang),         // isr_enet
  Some(isr_hang),         // isr_rit
  Some(isr_hang),         // isr_mcpwm
  Some(isr_hang),         // isr_qei
  Some(isr_hang),         // isr_pll_1
  Some(isr_hang),         // isr_usb_activity
  Some(isr_hang),         // isr_can_activity
];
