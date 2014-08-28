// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

/*!
SPI peripheral
*/

use hal::spi;

#[path="../../lib/wait_for.rs"] mod wait_for;

pub enum ChipSelect {
  CS0 = 0,
  CS1 = 1,
  CS2 = 2,
  CS3 = 3,
  CS4 = 4,
  CS5 = 5,
}

/// An SPI peripheral instance
pub struct DSPI {
  reg: &'static reg::DSPI,
  cs: ChipSelect,
}

impl DSPI {
  fn new(reg: &'static reg::DSPI, cs: ChipSelect) -> DSPI {
    reg.mcr.set_halt(false);
    DSPI {reg: reg, cs: cs}
  }
}

impl spi::SPI for DSPI {
  fn write(&self, value: u8) {
    wait_for!(self.reg.sr.tfff());
    self.reg.sr.clear_tfff();
    // TODO(bgamari): Need to ensure this doesn't read
    self.reg.pushr
      .set_txdata(value as u32)
      .set_pcs(self.cs as u32);
  }

  fn read(&self) -> u8 {
    wait_for!(self.reg.sr.rfdf());
    self.reg.popr.rxdata() as u8
  }
}

/// Registers
pub mod reg {
  use lib::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(DSPI = {
    0x0     => reg32 mcr { //! Module configuration register
      0      => halt,      //= Start/stop transfers
      8..9   => smpl_pt {  //! How many clocks between SCK edge and SIN sample
        0x0  => SMPL_PT_0_CLKS,
        0x1  => SMPL_PT_1_CLKS,
        0x2  => SMPL_PT_2_CLKS,
      }
      10     => clr_rxf,   //= Flush RX FIFO
      11     => clr_txf,   //= Flush TX FIFO
      12     => dis_rxf,   //= Disable receive FIFO
      13     => dis_txf,   //= Disable transmit FIFO
      14     => mdis,      //= Disable module
      15     => doze,      //= Doze enable
      16..20 => pcsis[5],  //= Chip select inactive states
      24     => rooe,      //= Receive FIFO overflow overwrite enable
      26     => mtfe,      //= Modified timing format enable
      27     => frz,       //= Freeze
      28..29 => dconf {    //! Peripheral configuration
        0x0  => SPI,
      }
      30     => cont_scke, //= Continuous SCK enable
      31     => mstr {     //! Master/slave mode select
        0x0  => SLAVE,
        0x1  => MASTER,
      }
    }

    0x8     => reg32 tcr { //! Transfer count register
      16..31 => tcnt,      //= Number of frames transmitted
    }

    0xc     => reg32 ctar[2] { //! Clock and transfer attributes
      /// Baud rate scaler (master only)
      ///
      /// SCK baud rate = (f_sys / PBR) * (1 + DBR) / BR
      0..3   => br,
      /// Delay after transfer scaler (master only)
      ///
      /// t_delay = (1/f_sys) * PDT * DT
      4..7   => dt,
      /// After SCK delay scaler (master only)
      ///
      /// t_ASC = (1/f_sys) * PASC * ASC
      8..11  => asc,
      /// CS to SCK delay scaler (master only)
      12..15 => cssck,
      /// Baud rate prescaler (master only)
      17..16 => pbr {
        0x0  => PBR_2,
        0x1  => PBR_3,
        0x2  => PBR_5,
        0x3  => PBR_7,
      }
      /// Delay after transfer prescaler (master only)
      18..19 => pdt {
        0x0  => PDT_2,
        0x1  => PDT_3,
        0x2  => PDT_5,
        0x3  => PDT_7,
      },
      /// After SCK delay prescaler (master only)
      20..21 => pasc {
        0x0  => PASC_2,
        0x1  => PASC_3,
        0x2  => PASC_5,
        0x3  => PASC_7,
      },
      /// CS to SCK delay prescaler (master only)
      22..23 => pcssck {
        0x0  => PCSSCK_2,
        0x1  => PCSSCK_3,
        0x2  => PCSSCK_5,
        0x3  => PCSSCK_7,
      },
      /// Transfer LSB first (master only)
      24     => lsbfe,
      /// Clock phase
      25     => cpha,
      /// Clock polarity
      26     => cpol,
      /// Frame size
      ///
      /// Number of bits transferred per frame minus one.
      /// In master mode the top bit of this field is the DBR, double
      /// baud rate, flag
      27..31 => fmsz,
    }

    0x2c    => reg32 sr {                 //! Status register
      0..3   => popnxtptr,                //= Pop next pointer
      4..7   => rxctr,                    //= RX FIFO counter
      8..11  => txnxtptr,                 //= Transmit next pointer
      12..15 => txctr,                    //= TX FIFO counter
      17     => rfdf: set_to_clear,       //= Receive FIFO drain flag
      19     => rfof: set_to_clear,       //= Receive FIFO overflow flag
      25     => tfff: set_to_clear,       //= Transmit FIFO fill flag
      27     => tfuf: set_to_clear,       //= Transmit FIFO underflow flag
      28     => eoqf: set_to_clear,       //= End of queue flag
      30     => rxrxs: set_to_clear,      //= TX/RX running
      31     => tcf: set_to_clear,        //= Transfer complete flag
    }

    0x30    => reg32 rser {     //! DMA/interrupt request select and enable
      /// Receive FIFO drain DMA/interrupt request select
      16     => rfdf_dirs {
        0x0  => RFDF_IRQ,
        0x1  => RFDF_DMA,
      }
      17     => rfdf_re,        //= Recieve FIFO drain request enable
      19     => rfof_re,        //= Receive FIFO overflow request enable
      /// Transmit FIFO fill DMA/interrupt request select
      24     => tfff_dirs {
        0x0  => TFFF_IRQ,
        0x1  => TFFF_DMA,
      }
      25     => tfff_re,        //= Transmit FIFO fill request enable
      27     => tfuf_re,        //= Transmit FIFO underflow request enable
      28     => eoqf_re,        //= End of Queue request enable
      31     => tcf_re,         //= Transmission complete request enable
    }

    0x34    => reg32 pushr {    //! TX FIFO register
      0..15  => txdata,         //= Transmitted data
      16..21 => pcs[6],         //= Chip select state (master only)
      /// Clear transfer counter (`TCR.TCNT`) (master only)
      26     => ctcnt,
      /// Set End of Queue flag (`SR.EOQF`) at end of transfer (master only)
      27     => eoq,
      28..30 => ctas,           //= CTAR select (master only)
      31     => cont,           //= Continuous CS enable (master only)
    }

    0x38    => reg32 popr {     //! RX FIFO register
      0..31  => rxdata: ro,     //= Received data
    }

    0x3c    => reg32 txfr[4] {  //! TX FIFO debug registers
      0..15  => txdata: ro,
      16..31 => txcmd_txdata: ro,
    }

    0x7c    => reg32 rxfr[4] {  //! RX FIFO debug registers
      0..31  => rxdata: ro,
    }
  })
}
