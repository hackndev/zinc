// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
// Based upon work by Ben Harris <mail@bharr.is>
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

use core::option::Option::{self, Some, None};

extern {
  fn isr_dma_0();
  fn isr_dma_1();
  fn isr_dma_2();
  fn isr_dma_3();
  fn isr_dma_4();
  fn isr_dma_5();
  fn isr_dma_6();
  fn isr_dma_7();
  fn isr_dma_8();
  fn isr_dma_9();
  fn isr_dma_10();
  fn isr_dma_11();
  fn isr_dma_12();
  fn isr_dma_13();
  fn isr_dma_14();
  fn isr_dma_15();
  fn isr_dma_err();
  fn isr_flash_complete();
  fn isr_flash_collision();
  fn isr_low_volt();
  fn isr_llwu();
  fn isr_wdt();
  fn isr_i2c_0();
  fn isr_i2c_1();
  fn isr_spi_0();
  fn isr_spi_1();
  fn isr_can_0_msg();
  fn isr_can_0_bus();
  fn isr_can_0_err();
  fn isr_can_0_tx();
  fn isr_can_0_rx();
  fn isr_can_0_wake();
  fn isr_i2s_0_tx();
  fn isr_i2s_0_rx();
  fn isr_uart_0_lon();
  fn isr_uart_0_stat();
  fn isr_uart_0_err();
  fn isr_uart_1_stat();
  fn isr_uart_1_err();
  fn isr_uart_2_stat();
  fn isr_uart_2_err();
  fn isr_adc_0();
  fn isr_adc_1();
  fn isr_cmp_0();
  fn isr_cmp_1();
  fn isr_cmp_2();
  fn isr_ftm_0();
  fn isr_ftm_1();
  fn isr_ftm_2();
  fn ist_cmt();
  fn isr_rtc_alarm();
  fn isr_rtc_tick();
  fn isr_pit_0();
  fn isr_pit_1();
  fn isr_pit_2();
  fn isr_pit_3();
  fn isr_pdb();
  fn isr_usb();
  fn isr_usb_dcd();
  fn isr_dac_0();
  fn isr_tsi();
  fn isr_mcg();
  fn isr_lptimer();
  fn isr_port_a();
  fn isr_port_b();
  fn isr_port_c();
  fn isr_port_d();
  fn isr_port_e();
  fn isr_soft();
}

#[link_section=".flash_configuration"]
#[allow(non_upper_case_globals)]
pub static FlashConfigField: [usize; 4] = [
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFE,
];

#[allow(non_upper_case_globals)]
const ISRCount: usize = 95;

#[link_section=".isr_vector_nvic"]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static NVICVectors: [Option<unsafe extern fn()>; ISRCount] = [
  Some(isr_dma_0),
  Some(isr_dma_1),
  Some(isr_dma_2),
  Some(isr_dma_3),
  Some(isr_dma_4),
  Some(isr_dma_5),
  Some(isr_dma_6),
  Some(isr_dma_7),
  Some(isr_dma_8),
  Some(isr_dma_9),
  Some(isr_dma_10),
  Some(isr_dma_11),
  Some(isr_dma_12),
  Some(isr_dma_13),
  Some(isr_dma_14),
  Some(isr_dma_15),
  Some(isr_dma_err),
  None,
  Some(isr_flash_complete),
  Some(isr_flash_collision),
  Some(isr_low_volt),
  Some(isr_llwu),
  Some(isr_wdt),
  None,
  Some(isr_i2c_0),
  Some(isr_i2c_1),
  Some(isr_spi_0),
  Some(isr_spi_1),
  None,
  Some(isr_can_0_msg),
  Some(isr_can_0_bus),
  Some(isr_can_0_err),
  Some(isr_can_0_tx),
  Some(isr_can_0_rx),
  Some(isr_can_0_wake),
  Some(isr_i2s_0_tx),
  Some(isr_i2s_0_rx),
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  Some(isr_uart_0_lon),
  Some(isr_uart_0_stat),
  Some(isr_uart_0_err),
  Some(isr_uart_1_stat),
  Some(isr_uart_1_err),
  Some(isr_uart_2_stat),
  Some(isr_uart_2_err),
  None,
  None,
  None,
  None,
  None,
  None,
  Some(isr_adc_0),
  Some(isr_adc_1),
  Some(isr_cmp_0),
  Some(isr_cmp_1),
  Some(isr_cmp_2),
  Some(isr_ftm_0),
  Some(isr_ftm_1),
  Some(isr_ftm_2),
  Some(ist_cmt),
  Some(isr_rtc_alarm),
  Some(isr_rtc_tick),
  Some(isr_pit_0),
  Some(isr_pit_1),
  Some(isr_pit_2),
  Some(isr_pit_3),
  Some(isr_pdb),
  Some(isr_usb),
  Some(isr_usb_dcd),
  None,
  None,
  None,
  None,
  None,
  None,
  Some(isr_dac_0),
  None,
  Some(isr_tsi),
  Some(isr_mcg),
  Some(isr_lptimer),
  None,
  Some(isr_port_a),
  Some(isr_port_b),
  Some(isr_port_c),
  Some(isr_port_d),
  Some(isr_port_e),
  None,
  None,
  Some(isr_soft),
];
