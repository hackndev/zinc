// Zinc, the bare metal stack for rust.
// Copyright 2014 Lionel Flandrin <lionel@svkt.org>
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

//! SPI module
//! Enables SPI interface on any of the 4 SSI (Synchronous Serial Interface) modules in TM4C microcontrollers

use hal::tiva_c::sysctl;
use util::support::get_reg_ref;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Clock source for SSI module
/// TODO(jamwaffles) Implement the ability to choose a clock source
// pub enum ClockSource {
//   /// System configured clock source
//   System = 0x0,

//   /// The Precision Internal Oscillator @16MHz
//   PIOSC  = 0x5,
// }

/// There are 4 SSI instances an SPI interface can use
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum SpiId {
  Spi0,
  Spi1,
  Spi2,
  Spi3,
}

/// Structure describing a single SPI interface
#[derive(Clone, Copy)]
pub struct Spi {
  /// SSI registers
  regs: &'static reg::Ssi,
}

impl Spi {
  /// Create and setup a UART.
  pub fn new(id: SpiId) -> Spi {

    let (periph, regs) = match id {
      SpiId::Spi0 => (sysctl::periph::ssi::SSI_0, reg::SSI_0),
      SpiId::Spi1 => (sysctl::periph::ssi::SSI_1, reg::SSI_1),
      SpiId::Spi2 => (sysctl::periph::ssi::SSI_2, reg::SSI_2),
      SpiId::Spi3 => (sysctl::periph::ssi::SSI_3, reg::SSI_3),
    };

    let spi = Spi { regs: get_reg_ref(regs) };

    periph.ensure_enabled();

    spi.configure();

    spi
  }

  /// Configure the SSI into SPI mode. Currently hard coded at 3.2MHz for 16MHz PIOSC
  fn configure(&self) {
    let sysclk = sysctl::clock::sysclk_get();

    self.regs.ssicr1
      .set_sse(false)
      .set_ms(false)
      .set_lbm(false);

    // 3.2MHz fixed bitrate for 16MHz crystal and 80MHz PLL
    self.regs.ssicpsr
      .set_cpsdvsr(25);

    self.regs.ssicr0
      .set_scr(0)
      .set_sph(false)
      .set_spo(false)
      .set_frf(0)     // SPI mode
      .set_dss(0x7);  // 8 bit frames

    // Turn on SSI now that we've configured it
    self.regs.ssicr1
      .set_sse(true);
  }

  /// Wait for SSI TX FIFO to be ready
  /// This checks the busy flag (0 = not busy) and the "transmit FIFO not full" flag (1 = not full)
  fn writeable(&self) -> bool {
    !self.regs.ssisr.bsy() && self.regs.ssisr.tnf()
  }

  /// Wait for SSI data buffer to be readable
  fn readable(&self) -> bool {
    !self.regs.ssisr.bsy()
  }
}

impl ::hal::spi::Spi for Spi {
  fn write(&self, value: u8) {
    wait_for!(self.writeable());

    self.regs.ssidr.set_data(value as u16);
  }

  fn read(&self) -> u8 {
    wait_for!(self.readable());

    self.regs.ssidr.data() as u8
  }
}

#[allow(missing_docs)]
pub mod reg {
  //! SSI registers definition
  use volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(Ssi = {
    /// SSI control register 0
    0x000 => reg16 ssicr0 {
      0..3 => dss: rw,    //= Data size select, 0x7 = 8 bits, see datasheet page 970 for other values
      4..5 => frf: rw,    //= Frame format, 0 = SPI 1 = TI SSF 2 Microwire
      6    => spo: rw,    //= SSI polarity, 0 = steady staate low on SSInCLK 1 = steady state high
      7    => sph: rw,    //= SSI clock phase, 1 = first edge 0 = second edge
      15..8 => scr: rw,   //= SSI clock rate, calculated from formula `SysClk / (CPSDVSR * (1 + SCR))`
    },

    /// SSI control register 1
    0x004 => reg16 ssicr1 {
      0 => lbm: rw,       //= Loopback enable, 1 = enabled 0 = normal operation
      1 => sse: rw,       //= Serial port enable 1 = enabled 0 = disabled. Must be cleared before SSI can be configured
      2 => ms: rw,        //= Master/slave select 0 = master 1 = slave
      4 => eot: rw,       //= End of transmission. Not sure what this is
    },

    /// SSI send/receive register
    /// When the SSIDR register is read, the entry in the receive FIFO that is pointed to by the current FIFO read pointer is accessed. When a data value is removed by the SSI receive logic from the incoming data frame, it is placed into the entry in the receive FIFO pointed to by the current FIFO write pointer.
    /// When the SSIDR register is written to, the entry in the transmit FIFO that is pointed to by the write pointer is written to. Data values are removed from the transmit FIFO one value at a time by the transmit logic. Each data value is loaded into the transmit serial shifter, then serially shifted out onto the SSInTx pin at the programmed bit rate.
    /// See page 973 in datasheet for more details
    0x008 => reg16 ssidr {
      0..15 => data: rw   //= Receive/transmit data
    },

    /// SSI status register
    0x00c => reg16 ssisr {
      0 => tfe: ro,       //= TX FIFO empty 1 = empty 0 = not empty
      1 => tnf: ro,       //= TX FIFO full 1 = not full 0 = full
      2 => rne: ro,       //= RX FIFO 1 = not empty 0 = empty
      3 => rff: ro,       //= RX FIFO full 0 = not full 1 = full
      4 => bsy: ro,       //= SSI busy 0 = idle, can send data 1 = transmitting, can't send data yet
    },

    /// SSI clock prescale register
    0x010 => reg16 ssicpsr {
      0..7 => cpsdvsr: rw //= SSI clock prescale divisor, 2 – 254 inclusive
    },

    /// SSI clock source configuration
    0xfc8 => reg16 ssicc {
      0..3 => cs: rw,     //= SSI baud clock source, 0x0 = system clock, 0x5 = PIOSC
    },
  });

  pub const SSI_0: *const Ssi = 0x40008000 as *const Ssi;
  pub const SSI_1: *const Ssi = 0x40009000 as *const Ssi;
  pub const SSI_2: *const Ssi = 0x4000A000 as *const Ssi;
  pub const SSI_3: *const Ssi = 0x4000B000 as *const Ssi;
}