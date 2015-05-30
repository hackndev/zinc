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

//! ISR Data for k20

use core::option::Option::{self, Some};

extern {
  fn isr_wdt();
  fn isr_timer_0();
  fn isr_timer_1();
  fn isr_timer_2();
  fn isr_timer_3();
  fn isr_uart_0();
  fn isr_uart_1();
  fn isr_uart_2();
  fn isr_uart_3();
  fn isr_pwm_1();
  fn isr_i2c_0();
  fn isr_i2c_1();
  fn isr_i2c_2();
  fn isr_spi();
  fn isr_ssp_0();
  fn isr_ssp_1();
  fn isr_pll_0();
  fn isr_rtc();
  fn isr_eint_0();
  fn isr_eint_1();
  fn isr_eint_2();
  fn isr_eint_3();
  fn isr_adc();
  fn isr_bod();
  fn isr_usb();
  fn isr_can();
  fn isr_dma();
  fn isr_i2s();
  fn isr_enet();
  fn isr_rit();
  fn isr_mcpwm();
  fn isr_qei();
  fn isr_pll_1();
  fn isr_usb_activity();
  fn isr_can_activity();
}

#[allow(non_upper_case_globals)]
const ISRCount: usize = 35;

#[allow(non_upper_case_globals)]
#[link_section=".isr_vector_nvic"]
#[no_mangle]
pub static NVICVectors: [Option<unsafe extern fn()>; ISRCount] = [
  // s.a. lpc17xx user manual, table 50 (chapter 6.3)
  Some(isr_wdt),
  Some(isr_timer_0),
  Some(isr_timer_1),
  Some(isr_timer_2),
  Some(isr_timer_3),
  Some(isr_uart_0),
  Some(isr_uart_1),
  Some(isr_uart_2),
  Some(isr_uart_3),
  Some(isr_pwm_1),
  Some(isr_i2c_0),
  Some(isr_i2c_1),
  Some(isr_i2c_2),
  Some(isr_spi),
  Some(isr_ssp_0),
  Some(isr_ssp_1),
  Some(isr_pll_0),
  Some(isr_rtc),
  Some(isr_eint_0),
  Some(isr_eint_1),
  Some(isr_eint_2),
  Some(isr_eint_3),
  Some(isr_adc),
  Some(isr_bod),
  Some(isr_usb),
  Some(isr_can),
  Some(isr_dma),
  Some(isr_i2s),
  Some(isr_enet),
  Some(isr_rit),
  Some(isr_mcpwm),
  Some(isr_qei),
  Some(isr_pll_1),
  Some(isr_usb_activity),
  Some(isr_can_activity),
];
