#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Available UART peripherals.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum SPIPeripheral {
  SPI1,
  SPI2,
  SPI3,
  SPI4,
  SPI5,
  SPI6
}

/// Structure describing a UART instance.
#[derive(Clone, Copy)]
pub struct SPI {
  reg: &'static reg::SPI,
}

impl SPIPeripheral {
  fn reg(self) -> &'static reg::SPI {
    match self {
      SPIPeripheral::SPI1 => &reg::SPI1,
      SPIPeripheral::SPI2 => &reg::SPI2,
      SPIPeripheral::SPI3 => &reg::SPI3,
      SPIPeripheral::SPI4 => &reg::SPI4,
      SPIPeripheral::SPI5 => &reg::SPI5,
      SPIPeripheral::SPI6 => &reg::SPI6,
    }
  }
}

//impl UART {
  /// Returns platform-specific UART object that implements CharIO trait.
  /*pub fn new(peripheral: UARTPeripheral, baudrate:  u32, word_len: u8,
      parity: uart::Parity, stop_bits: u8) -> UART {
    let uart = UART {
      reg: peripheral.reg()
    };
    uart.set_baud_rate(baudrate);
    uart.set_mode(reg::UART_c1_m::from_u8(word_len), parity, StopBit::from_u8(stop_bits));
    uart.set_fifo_enabled(true);

    uart
  }

  fn uart_clock(&self) -> u32 {
    48000000 // FIXME(bgamari): Use peripheral clocks
  }

  fn set_baud_rate(&self, baud_rate: u32) {
    let sbr: u32 = self.uart_clock() / 16 / baud_rate;
    let brfa: u32 = (2 * self.uart_clock() / baud_rate) % 32;
    (*self.reg).bdh.set_sbr((sbr >> 8) as u8);
    (*self.reg).bdl.set_sbr((sbr & 0xff) as u8);
    (*self.reg).c4.set_brfa(brfa as u8);
  }

  fn set_mode(&self, word_len: reg::UART_c1_m, parity: uart::Parity,
              _stop_bits: StopBit) {
    use hal::uart::Parity::*;
    let mut c1 = (*self.reg).c1.set_m(word_len);
    match parity {
      Disabled => {c1.set_pe(false);}
      Odd      => {c1.set_pe(true).set_pt(reg::UART_c1_pt::Odd);}
      Even     => {c1.set_pe(true).set_pt(reg::UART_c1_pt::Even);}
      Forced1  => unsafe { abort() },
      Forced0  => unsafe { abort() },
    };
    (*self.reg).c2.set_te(true).set_re(true);
  }

  fn set_fifo_enabled(&self, enabled: bool) {
    (*(self.reg)).pfifo.set_rxfe(enabled).set_txfe(enabled);
  }*/
//}

/// Register definitions
pub mod reg {
  use volatile_cell::VolatileCell;

  ioregs!(SPI = {
    0x0 => reg32 cr1 {  //! spi config regiser 1
        0 => cpha,       //= Clock_Phase (0: fst clock transition is data capture edge, 1: second)
        1 => cpol,   //= Clock polarity (0:CK to 0 when idle, 1: 1)
        2 => mstr,    //= Master selection (1: master, 0: slave)
        3..5 => br {
            0x0 => SPICLockDivider32,
            0x1 => SPICLockDivider64,
            0x2 => SPICLockDivider128,
            0x3 => SPICLockDivider256
        },
        6 => spe, //=  SPI enable
        7 => lsbfirst, //= Frame format (0: MSB, 1: LSB)
        8 => ssi, //= Internal slavec select
        9 => ssm, //= Slave software management
        10 => rxonly, //= Receive only neable
        11 => dff, //= Data frame format(0: 8bits, 1: 16bits)
        12 => crcnext, //= CRC transfert next
        13 => crcen, //= Hardware CRC enable
        14 => bidiode, //= Output enable in bidirectional mode
        15 => bidimode //= Bidirectional datamode enable
    },
    0x04 => reg32 cr2 { //! spi config regiser 2
        0 => rxdmaen, //= RX buffer DMA enable
        1 => txdmaen, //= Tx buffer DMA enable
        2 => ssoe, //= SS output enable
        //= 3 is reserved
        4 => frf, //= Frame format (0: SPI Motorola mode, 1: SPI TI mode)
        5 => errie, //= Error interrupt enable
        6 => rxneie, //=Rx buffer not empty interrupt enable
        7 => txneie //= Tx buffer empty intterup enable
    },
    0x08 => reg32 sr { //! spi status register
        0 => rxne, //= Receive buffer not empty
        1 => txe, //= Transmit buffer empty
        2 => chside, //= Channel side
        3 => udr, //= underrun flag
        4 => crcerr, //= CRC error flag
        5 => modf, //= Mode fault
        6 => ovr, //= Overrun flag
        7 => bsy, //Busy flag
        8 => fre //= Frame format error (0: No frame format error)
    },
    0x0c => reg32 dr { //! spi data register
        0..15 => dr //= data
    },
    0x10 => reg32 crcpr { //! spi
        0..15 => crcpoly //= data
    },
    0x14 => reg32 rxcrcr { //! spi
        0..15 => rxcrc //= data
    },
    0x18 => reg32 txcrcr { //! spi
        0..15 => txcrc //= data
    }
    //TODO: i2s ?
  });

  extern {
    #[link_name="stm32f4_iomem_SPI1"] pub static SPI1: SPI;
    #[link_name="stm32f4_iomem_SPI2"] pub static SPI2: SPI;
    #[link_name="stm32f4_iomem_SPI3"] pub static SPI3: SPI;
    #[link_name="stm32f4_iomem_SPI4"] pub static SPI4: SPI;
    #[link_name="stm32f4_iomem_SPI5"] pub static SPI5: SPI;
    #[link_name="stm32f4_iomem_SPI6"] pub static SPI6: SPI;
  }
}
