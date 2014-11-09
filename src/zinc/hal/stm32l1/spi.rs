// Zinc, the bare metal stack for rust.
// Copyright 2014 Dzmitry "kvark" Malyshau <kvarkus@gmail.com>
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

//! Serial Peripheral Interface for STM32L1.

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(SPI = {
    0x00 => reg16 cr1 { // control 1
      0 => clock_phase : rw,
      1 => clock_polarity : rw,
      2 => master : rw,
      5..3 => baud_rate : rw,
      6 => spi_enable : rw,
      7 => frame_format : rw,
      8 => internal_slave_select : rw,
      9 => software_slave_management : rw,
      10 => receive_only : rw,
      11 => data_frame_format : rw,
      12 => transmit_crc_next : rw,
      13 => hardware_crc_enable : rw,
      14 => bidirectional_output_enable : rw,
      15 => bidirectional_data_mode : rw,
    },
    0x04 => reg8 cr2 { // control 2
      0 => rx_dma_enable : rw,
      1 => tx_dma_enable : rw,
      2 => ss_output_enable : rw,
      3 => frame_format : rw,
      // 4 is reserved
      5 => error_interrupt_enable : rw,
      6 => rx_buffer_not_empty_interrupt_enable : rw,
      7 => tx_buffer_empty_interrupt_enable : rw,
    },
    0x08 => reg8 sr { // status
      0 => receive_buffer_not_empty : ro,
      1 => transmit_buffer_empty : ro,
      2 => channel_side : ro,
      3 => underrun_flag : ro,
      4 => crc_error : ro,
      5 => mode_fault : ro,
      6 => overrun_flag : ro,
      7 => busy_flag : ro,
    },
    0x0C => reg16 dr { // data
      15..0 => data : rw,
    },
    0x10 => reg16 crc { // CRC
      15..0 => crc : rw,
    },
    0x14 => reg16 rx_crc { // Rx CRC
      15..0 => crc : rw,
    },
    0x18 => reg16 tx_crc { // Tx CRC
      15..0 => crc : rw,
    },
    0x1C => reg16 i2s_cfgr { // I2S config
      15..0 => config : rw,
    },
    0x20 => reg16 i2c_pr { // I2S prescaler
      15..0 => prescaler : rw,
    },
  })

  extern {
    #[link_name="stm32l1_iomem_SPI1"] pub static SPI1: SPI;
    #[link_name="stm32l1_iomem_SPI2"] pub static SPI2: SPI;
    #[link_name="stm32l1_iomem_SPI3"] pub static SPI3: SPI;
  }
}
