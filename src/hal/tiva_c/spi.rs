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

//! SPI configuration

/// Enables SPI interface on any of the 4 SSI (Synchronous Serial Interface)
/// modules in TM4C microcontrollers

use core::intrinsics::abort;
use hal::tiva_c::sysctl;
use util::support::get_reg_ref;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// There are 4 SSI instances an SPI interface can use
/// See the TM4C123GH6PM datasheet page 954 for detailed signal to pin mappings
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum SpiId {
  /// SPI over SSI0, uses pins PA2:5
  Spi0,

  /// SPI over SSI1, uses pins PF0:3 or PD0:3
  Spi1,

  /// SPI over SSI2, uses pins PB4:7
  Spi2,

  /// SPI over SSI3, uses pins PD0:3
  Spi3,
}

/**
SPI configuration object

Configuration object used when instantiating an SPI instance

Note: The SPI GPIO pins must be set up correctly before using SPI. For example,
for a master-only TX and CK config on SSI0, the platformtree section might look
like the following (taken from a Tiva C configuration):

```
gpio {
  a {
    spi_ck@2 {
      direction = "out";
      function  = 2;
    }

    spi_tx@5 {
      direction = "out";
      function  = 2;
    }
  }
}
```
*/
pub struct SpiConf {
  /// Which SPI peripheral to use. The Tiva C has 4 SSI interfaces that can be used for SPI
  pub peripheral: SpiId,

  /// Bus frequency in Hz. Peripheral clock scaling is calculated from this value to a possibly non-exact match
  pub frequency: u32,
}

/**
Structure describing a single SPI interface

# Examples

Create an SPI interface on SSI0 with a run frequency of 4MHz

```
#![crate_type = "staticlib"]
#![feature(plugin, start, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;

use zinc::hal::spi::Spi;
use zinc::hal::tiva_c::spi;

platformtree!(
  tiva_c@mcu {
    // Tiva C ends up with an 80MHz clock from 16MHz external xtal and x5 PLL
    clock {
      source = "MOSC";
      xtal   = "X16_0MHz";
      pll    = true;
      div    = 5;
    }

    gpio {
      a {
        spi_ck@2 {
          direction = "out";
          function  = 2;
        }

        spi_tx@5 {
          direction = "out";
          function  = 2;
        }
      }
    }
  }

  os {
    single_task {
      loop = "run";
    }
  }
);

fn run() {
  let spi = spi::Spi::new(spi::SpiConf {
    peripheral: spi::SpiId::Spi0,
    frequency: 4_000_000
  });

  loop {
    spi.write('a' as u8);
  }
}
```
*/
#[derive(Clone, Copy)]
pub struct Spi {
  /// SSI registers
  regs: &'static reg::Ssi,
}

impl Spi {
  /// Create and setup an SPI interface.
  pub fn new(config: SpiConf) -> Spi {

    let (periph, regs) = match config.peripheral {
      SpiId::Spi0 => (sysctl::periph::ssi::SSI_0, reg::SSI_0),
      SpiId::Spi1 => (sysctl::periph::ssi::SSI_1, reg::SSI_1),
      SpiId::Spi2 => (sysctl::periph::ssi::SSI_2, reg::SSI_2),
      SpiId::Spi3 => (sysctl::periph::ssi::SSI_3, reg::SSI_3),
    };

    let spi = Spi { regs: get_reg_ref(regs) };

    // Make sure peripheral clock gating is enabled
    periph.ensure_enabled();

    spi.configure(config);

    spi
  }

  /// Configure the SSI into SPI mode
  fn configure(&self, config: SpiConf) {
    // Disable peripheral so we can configure it
    self.regs.ssicr1.set_sse(false);

    self.regs.ssicr1
      .set_sse(false)
      .set_ms(false)
      .set_lbm(false);

    // Set clock rate
    self.set_frequency(config.frequency);

    self.regs.ssicr0
      .set_sph(false)
      .set_spo(false)
      .set_frf(0)     // Put SSI into SPI mode
      .set_dss(0x7);  // 8 bit frames

    // Turn on peripheral now that we've configured it
    self.regs.ssicr1.set_sse(true);
  }

  /// Set SPI frequency
  ///
  /// This function computes the divisor and exponent (for lack of a better name).
  /// These are stored in `ssicpsr.cpsdvsr` and `ssicr0.scr` respectively.
  /// The clock rate formula (taken from the datasheet is)
  /// `ClockRate = SysClk / (CPSDVSR * (1 + SCR))` where SysClk is the system clock in Hz.
  fn set_frequency(&self, freq: u32) {
    let sysclk = sysctl::clock::sysclk_get() as u32;

    let mut divisor: u32 = 2;

    while divisor <= 254 {
      let prescale_hz = sysclk / divisor;

      // Calculate exponent
      // TODO(jamwaffles) The below line crashes my Tiva,
      // presumably because floating point is broken?
      // let scr = ((prescale_hz as f32 / freq as f32) + 0.5f32) as u32;

      let scr = prescale_hz / freq;

      // Check we can support the divider
      if scr < 256 {
        self.regs.ssicpsr.set_cpsdvsr(divisor as u16);
        self.regs.ssicr0.set_scr((scr - 1) as u16);

        return
      }

      divisor += 2;
    }

    // TODO(jamwaffles): Return a Result from here as well as the calling
    // configure method
    unsafe { abort() };
  }

  /// Wait for SSI TX FIFO to be ready
  /// This checks the busy flag (0 = not busy) and the "transmit FIFO not full"
  /// flag (1 = not full)
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
      0..3 => dss: rw,    //= Data size select, 0x7 = 8 bits, see datasheet page
                          //= 970 for other values
      4..5 => frf: rw,    //= Frame format, 0 = SPI 1 = TI SSF 2 Microwire
      6    => spo: rw,    //= SSI polarity, 0 = steady staate low on
                          //= SSInCLK 1 = steady state high
      7    => sph: rw,    //= SSI clock phase, 1 = first edge 0 = second edge
      15..8 => scr: rw,   //= SSI clock rate, calculated from formula
                          //= `SysClk / (CPSDVSR * (1 + SCR))`
    },

    /// SSI control register 1
    0x004 => reg16 ssicr1 {
      0 => lbm: rw,       //= Loopback enable, 1 = enabled 0 = normal operation
      1 => sse: rw,       //= Serial port enable 1 = enabled 0 = disabled.
                          //= Must be cleared before SSI can be configured
      2 => ms: rw,        //= Master/slave select 0 = master 1 = slave
      4 => eot: rw,       //= End of transmission. Not sure what this is
    },

    /// SSI send/receive register
    ///
    /// When the SSIDR register is read, the entry in the receive FIFO that is
    /// pointed to by the current FIFO read pointer is accessed. When a data
    /// value is removed by the SSI receive logic from the incoming data frame,
    /// it is placed into the entry in the receive FIFO pointed to by the
    /// current FIFO write pointer.
    ///
    /// When the SSIDR register is written to, the entry in the transmit FIFO
    /// that is pointed to by the write pointer is written to. Data values are
    /// removed from the transmit FIFO one value at a time by the transmit
    /// logic. Each data value is loaded into the transmit serial shifter, then
    /// serially shifted out onto the SSInTx pin at the programmed bit rate.
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
      4 => bsy: ro,       //= SSI busy 0 = idle, can send data 1 = transmitting,
                          //= can't send data yet
    },

    /// SSI clock prescale register
    0x010 => reg16 ssicpsr {
      0..7 => cpsdvsr: rw //= SSI clock prescale divisor, 2 – 254 inclusive
    },

    /// SSI clock source configuration
    0xfc8 => reg16 ssicc {
      0..3 => cs: rw,     //= SSI baud clock source, 0x0 = system clock,
                          //= 0x5 = PIOSC
    },
  });

  // TODO(jamwaffles): Change to placement ioregs
  pub const SSI_0: *const Ssi = 0x40008000 as *const Ssi;
  pub const SSI_1: *const Ssi = 0x40009000 as *const Ssi;
  pub const SSI_2: *const Ssi = 0x4000A000 as *const Ssi;
  pub const SSI_3: *const Ssi = 0x4000B000 as *const Ssi;
}