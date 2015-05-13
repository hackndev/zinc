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

use core::result::Result;
use core::result::Result::{Ok, Err};
use core::marker::Copy;

#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Available SPI peripherals.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Peripheral {
  Spi1,
  Spi2,
  Spi3,
}

/// SPI direction modes.
#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
  /// 2 lines, default mode
  FullDuplex,
  /// 2 lines, but read-only
  RxOnly,
  /// 1 line, read
  Rx,
  /// 1 line, transmit
  Tx,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone)]
pub enum Role {
  Slave = 0,
  Master = 1,
}

impl Copy for Role {}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum DataSize {
  U8 = 0,
  U16 = 1,
}

/// SPI data format.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum DataFormat {
  /// Most Significant Bit
  MsbFirst = 0,
  /// Least Significant Bit
  LsbFirst = 1,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ClockPhase {
  Edge1 = 0,
  Edge2 = 1,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ClockPolarity {
  Low = 0,
  High = 1,
}

/// SPI initialization errors.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Error {
  /// Invalid baud rate shift.
  BaudRate,
  /// Invalid resulting mode.
  Mode,
}

/// Structure describing a SPI instance.
#[derive(Clone, Copy)]
pub struct Spi {
  reg: &'static reg::SPI,
}

impl Spi {
  /// Create a new SPI port.
  pub fn new(peripheral: Peripheral, direction: Direction,
             role: Role, data_size: DataSize, format: DataFormat,
             prescaler_shift: u8) -> Result<Spi, Error> {
    use hal::stm32l1::peripheral_clock as clock;

    let (reg, clock) = match peripheral {
      Peripheral::Spi1 => (&reg::SPI1, clock::Apb2(clock::BusApb2::Spi1)),
      Peripheral::Spi2 => (&reg::SPI2, clock::Apb1(clock::BusApb1::Spi2)),
      Peripheral::Spi3 => (&reg::SPI3, clock::Apb1(clock::BusApb1::Spi3)),
    };

    clock.enable();

    // set direction
    reg.cr1.set_receive_only(direction == Direction::RxOnly);
    reg.cr1.set_bidirectional_data_mode(direction == Direction::Rx
        || direction == Direction::Tx);
    reg.cr1.set_bidirectional_output_enable(direction == Direction::Tx);

    // set role
    reg.cr1.set_master(role as usize != 0);
    reg.cr1.set_internal_slave_select(role as usize != 0);
    reg.cr1.set_software_slave_management(true);
    reg.cr2.set_ss_output_enable(false);

    // set data size and format (MSB or LSB)
    reg.cr1.set_data_frame_format(data_size as usize != 0);
    reg.cr1.set_frame_format(format as usize != 0);

    // set baud rate
    if prescaler_shift<1 || prescaler_shift>8 {
      return Err(Error::BaudRate)
    }
    reg.cr1.set_baud_rate(prescaler_shift as u16 - 1);

    // set clock mode
    reg.cr1.set_clock_phase(ClockPhase::Edge1 as usize != 0);
    reg.cr1.set_clock_polarity(ClockPolarity::Low as usize != 0);

    reg.i2s_cfgr.set_enable(false);
    reg.cr1.set_hardware_crc_enable(false);
    reg.crc.set_polynomial(0); //TODO

    if reg.sr.mode_fault() {
      Err(Error::Mode)
    }else {
      reg.cr1.set_spi_enable(true);
      Ok(Spi {
        reg: reg,
      })
    }
  }

  /// Returns the status byte.
  pub fn get_status(&self) -> u8 {
    //self.reg.sr.raw() //TODO(kvark): #245 doesn't work
    let mut r = 0u8;
    if self.reg.sr.receive_buffer_not_empty() {
      r |= 1u8<<0;
    }
    if self.reg.sr.transmit_buffer_empty() {
      r |= 1u8<<1;
    }
    if self.reg.sr.channel_side() {
      r |= 1u8<<2;
    }
    if self.reg.sr.underrun_flag() {
      r |= 1u8<<3;
    }
    if self.reg.sr.crc_error() {
      r |= 1u8<<4;
    }
    if self.reg.sr.mode_fault() {
      r |= 1u8<<5;
    }
    if self.reg.sr.overrun_flag() {
      r |= 1u8<<6;
    }
    if self.reg.sr.busy_flag() {
      r |= 1u8<<7;
    }
    r
  }
}

impl ::hal::spi::Spi for Spi {
  fn write(&self, value: u8) {
    wait_for!(self.reg.sr.transmit_buffer_empty());
    self.reg.dr.set_data(value as u16);
  }

  fn read(&self) -> u8 {
    wait_for!(self.reg.sr.receive_buffer_not_empty());
    self.reg.dr.data() as u8
  }
}

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
      15..0 => polynomial : rw,
    },
    0x14 => reg16 rx_crc { // Rx CRC
      15..0 => crc : rw,
    },
    0x18 => reg16 tx_crc { // Tx CRC
      15..0 => crc : rw,
    },
    0x1C => reg16 i2s_cfgr { // I2S config
      0 => channel_length : rw,
      2..1 => data_length : rw,
      3 => clock_polarity : rw,
      5..4 => standard_selection : rw,
      7 => pcm_frame_sync : rw,
      9..8 => configuration_mode : rw,
      10 => enable : rw,
      11 => mode_selection : rw,
    },
    0x20 => reg16 i2s_pr { // I2S prescaler
      7..0 => linear_prescaler : rw,
      8 => odd_factor : rw,
      9 => master_clock_output_enable : rw,
    },
  });

  extern {
    #[link_name="stm32l1_iomem_SPI1"] pub static SPI1: SPI;
    #[link_name="stm32l1_iomem_SPI2"] pub static SPI2: SPI;
    #[link_name="stm32l1_iomem_SPI3"] pub static SPI3: SPI;
  }
}
