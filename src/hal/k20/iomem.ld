/* Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Harris <mail@bharr.is>
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
*/

INCLUDE ./src/hal/cortex_m3/armmem.ld

PROVIDE(isr_dma_0           = isr_default_fault);
PROVIDE(isr_dma_1           = isr_default_fault);
PROVIDE(isr_dma_2           = isr_default_fault);
PROVIDE(isr_dma_3           = isr_default_fault);
PROVIDE(isr_dma_4           = isr_default_fault);
PROVIDE(isr_dma_5           = isr_default_fault);
PROVIDE(isr_dma_6           = isr_default_fault);
PROVIDE(isr_dma_7           = isr_default_fault);
PROVIDE(isr_dma_8           = isr_default_fault);
PROVIDE(isr_dma_9           = isr_default_fault);
PROVIDE(isr_dma_10          = isr_default_fault);
PROVIDE(isr_dma_11          = isr_default_fault);
PROVIDE(isr_dma_12          = isr_default_fault);
PROVIDE(isr_dma_13          = isr_default_fault);
PROVIDE(isr_dma_14          = isr_default_fault);
PROVIDE(isr_dma_15          = isr_default_fault);
PROVIDE(isr_dma_err         = isr_default_fault);
PROVIDE(isr_flash_complete  = isr_default_fault);
PROVIDE(isr_flash_collision = isr_default_fault);
PROVIDE(isr_low_volt        = isr_default_fault);
PROVIDE(isr_llwu            = isr_default_fault);
PROVIDE(isr_wdt             = isr_default_fault);
PROVIDE(isr_i2c_0           = isr_default_fault);
PROVIDE(isr_i2c_1           = isr_default_fault);
PROVIDE(isr_spi_0           = isr_default_fault);
PROVIDE(isr_spi_1           = isr_default_fault);
PROVIDE(isr_can_0_msg       = isr_default_fault);
PROVIDE(isr_can_0_bus       = isr_default_fault);
PROVIDE(isr_can_0_err       = isr_default_fault);
PROVIDE(isr_can_0_tx        = isr_default_fault);
PROVIDE(isr_can_0_rx        = isr_default_fault);
PROVIDE(isr_can_0_wake      = isr_default_fault);
PROVIDE(isr_i2s_0_tx        = isr_default_fault);
PROVIDE(isr_i2s_0_rx        = isr_default_fault);
PROVIDE(isr_uart_0_lon      = isr_default_fault);
PROVIDE(isr_uart_0_stat     = isr_default_fault);
PROVIDE(isr_uart_0_err      = isr_default_fault);
PROVIDE(isr_uart_1_stat     = isr_default_fault);
PROVIDE(isr_uart_1_err      = isr_default_fault);
PROVIDE(isr_uart_2_stat     = isr_default_fault);
PROVIDE(isr_uart_2_err      = isr_default_fault);
PROVIDE(isr_adc_0           = isr_default_fault);
PROVIDE(isr_adc_1           = isr_default_fault);
PROVIDE(isr_cmp_0           = isr_default_fault);
PROVIDE(isr_cmp_1           = isr_default_fault);
PROVIDE(isr_cmp_2           = isr_default_fault);
PROVIDE(isr_ftm_0           = isr_default_fault);
PROVIDE(isr_ftm_1           = isr_default_fault);
PROVIDE(isr_ftm_2           = isr_default_fault);
PROVIDE(ist_cmt             = isr_default_fault);
PROVIDE(isr_rtc_alarm       = isr_default_fault);
PROVIDE(isr_rtc_tick        = isr_default_fault);
PROVIDE(isr_pit_0           = isr_default_fault);
PROVIDE(isr_pit_1           = isr_default_fault);
PROVIDE(isr_pit_2           = isr_default_fault);
PROVIDE(isr_pit_3           = isr_default_fault);
PROVIDE(isr_pdb             = isr_default_fault);
PROVIDE(isr_usb             = isr_default_fault);
PROVIDE(isr_usb_dcd         = isr_default_fault);
PROVIDE(isr_dac_0           = isr_default_fault);
PROVIDE(isr_tsi             = isr_default_fault);
PROVIDE(isr_mcg             = isr_default_fault);
PROVIDE(isr_lptimer         = isr_default_fault);
PROVIDE(isr_port_a          = isr_default_fault);
PROVIDE(isr_port_b          = isr_default_fault);
PROVIDE(isr_port_c          = isr_default_fault);
PROVIDE(isr_port_d          = isr_default_fault);
PROVIDE(isr_port_e          = isr_default_fault);
PROVIDE(isr_soft            = isr_default_fault);

iomem_PERIPH0   = 0x40000000;

iomem_CROSSBAR  = 0x40004000;

iomem_DMACON    = 0x40008000;
iomem_DMATCD    = 0x40009000;

iomem_FLASHCON  = 0x4001F000;
iomem_FLASH     = 0x40020000;
iomem_DMAMUX0   = 0x40021000;

iomem_CAN0      = 0x40024000;

iomem_SPI0      = 0x4002C000;
iomem_SPI1      = 0x4002D000;
iomem_I2S0      = 0x4002F000;

iomem_CRC       = 0x40032000;

iomem_USBDCD    = 0x40035000;
iomem_PDB       = 0x40036000;
iomem_PIT       = 0x40037000;
iomem_FTM0      = 0x40038000;
iomem_FTM1      = 0x40039000;

iomem_ADC0      = 0x4003B000;

iomem_RTC       = 0x4003D000;
iomem_VBAT      = 0x4003E000;

iomem_LPTMR     = 0x40040000;
iomem_SYSREG    = 0x40041000;

iomem_TSI       = 0x40045000;

iomem_SIMLP     = 0x40047000;
iomem_SIM       = 0x40048000;
iomem_MUXA      = 0x40049000;
iomem_MUXB      = 0x4004A000;
iomem_MUXC      = 0x4004B000;
iomem_MUXD      = 0x4004C000;
iomem_MUXE      = 0x4004D000;

iomem_WDINT     = 0x40052000;

iomem_WDEXT     = 0x40061000;
iomem_CMT       = 0x40062000;

iomem_MCG       = 0x40064000;
iomem_OSC       = 0x40065000;
iomem_I2C0      = 0x40066000;
iomem_I2C1      = 0x40067000;

iomem_UART0     = 0x4006A000;
iomem_UART1     = 0x4006B000;
iomem_UART2     = 0x4006C000;

iomem_USB       = 0x40072000;
iomem_CMP       = 0x40073000;
iomem_VREF      = 0x40074000;

iomem_LLWU      = 0x4007C000;
iomem_PMC       = 0x4007D000;
iomem_SMC       = 0x4007E000;
iomem_RCM       = 0x4007F000;


iomem_PERIPH1   = 0x40080000;

iomem_FTM2      = 0x400B8000;

iomem_ADC1      = 0x400BB000;

iomem_DAC0      = 0x400CC000;
