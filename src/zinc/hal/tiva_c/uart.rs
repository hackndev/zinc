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

//! UART configuration

use hal::tiva_c::sysctl;
use util::support::get_reg_ref;

use drivers::chario::CharIO;
use hal::uart;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// There are 8 UART instances in total
#[allow(missing_docs)]
#[derive(Copy)]
pub enum UartId {
  Uart0,
  Uart1,
  Uart2,
  Uart3,
  Uart4,
  Uart5,
  Uart6,
  Uart7,
}

/// Structure describing a single UART
#[derive(Copy)]
pub struct Uart {
  /// UART register interface
  regs: &'static reg::Uart,
}

impl Uart {
  /// Create and setup a UART.
  pub fn new(id:        UartId,
             baudrate:  usize,
             word_len:  u8,
             parity:    uart::Parity,
             stop_bits: u8) -> Uart {

    let (periph, regs) = match id {
      UartId::Uart0 => (sysctl::periph::uart::UART_0, reg::UART_0),
      UartId::Uart1 => (sysctl::periph::uart::UART_1, reg::UART_1),
      UartId::Uart2 => (sysctl::periph::uart::UART_2, reg::UART_2),
      UartId::Uart3 => (sysctl::periph::uart::UART_3, reg::UART_3),
      UartId::Uart4 => (sysctl::periph::uart::UART_4, reg::UART_4),
      UartId::Uart5 => (sysctl::periph::uart::UART_5, reg::UART_5),
      UartId::Uart6 => (sysctl::periph::uart::UART_6, reg::UART_6),
      UartId::Uart7 => (sysctl::periph::uart::UART_7, reg::UART_7),
    };

    let uart = Uart { regs: get_reg_ref(regs) };

    periph.ensure_enabled();

    uart.configure(baudrate, word_len, parity, stop_bits);

    uart
  }

  /// Configure the UART
  fn configure(&self,
               baudrate:  usize,
               word_len:  u8,
               parity:    uart::Parity,
               stop_bits: u8) {
    let sysclk = sysctl::clock::sysclk_get();

    // compute the baud rate divisor rounded to the nearest
    let brd = ((((sysclk / 16) << 6) + baudrate / 2) / baudrate) as u32;

    self.regs.ctl
      // Disable the UART before configuration
      .set_uarten(false)
      // Enable TX
      .set_txe(true)
      // Enable TX
      .set_rxe(false)
      // Disable High-Speed
      .set_hse(false);

    self.regs.ibrd.set_divint(brd >> 6);
    self.regs.fbrd.set_divfrac(brd & ((1 << 6) - 1));

    let ( parity_en, even_parity, sticky_parity ) = match parity {
      uart::Parity::Disabled => (false, false, false),
      uart::Parity::Odd      => (true,  false, false),
      uart::Parity::Even     => (true,  true,  false),
      uart::Parity::Forced1  => (true,  false, true ),
      uart::Parity::Forced0  => (true,  true,  true ),
    };

    let wlen = match word_len {
      5 => reg::Uart_crh_wlen::Wlen5,
      6 => reg::Uart_crh_wlen::Wlen6,
      7 => reg::Uart_crh_wlen::Wlen7,
      8 => reg::Uart_crh_wlen::Wlen8,
      _ => panic!("Unsupported word length"),
    };

    self.regs.crh
      .set_wlen(wlen)
      .set_pen(parity_en)
      .set_eps(even_parity)
      .set_sps(sticky_parity)
      .set_stp2(stop_bits > 1);

    // Enable the UART
    self.regs.ctl.set_uarten(true);
  }
}

impl CharIO for Uart {
  fn putc(&self, value: char) {

    wait_for!(!self.regs.fr.txff());

    self.regs.data.set_data(value as u32);
  }
}

pub mod reg {
  //! Uart registers definition
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(Uart = {
    0x00 => reg32 data {
      0..11 => data,     //= RX/TX fifo data
    }
    0x18 => reg32 fr {
      0     => ctx:  ro, //= clear-to-send signal is asserted
      3     => busy: ro, //= UART is busy
      4     => rxfe: ro, //= RX FIFO is empty
      5     => txff: ro, //= TX FIFO is full
      6     => rxff: ro, //= RX FIFO is full
      7     => txfe: ro, //= TX FIFO is empty
    }
    0x24 => reg32 ibrd {
      0..15 => divint,   //= Baud-Rate divisor (integer part)
    }
    0x28 => reg32 fbrd {
      0..5  => divfrac,  //= Baud-Rate divisor (fractional part)
    }
    0x2C => reg32 crh {
      0     => brk,      //= UART send break
      1     => pen,      //= UART parity enable
      2     => eps,      //= UART even parity select
      3     => stp2,     //= UART two stop bits select
      4     => fen,      //= UART enable FIFOs
      5..6  => wlen {    //! UART world length
        0 => Wlen5,
        1 => Wlen6,
        2 => Wlen7,
        3 => Wlen8,
      }
      7     => sps,      //= UART stick parity select
    }
    0x30 => reg32 ctl {
      0     => uarten,   //= UART enable
      1     => siren,    //= UART SIR enable
      2     => sirlp,    //= UART SIR low-power mode
      3     => smart,    //= ISO 7816 Smart Card mode
      4     => eot,      //= End-of-Transmission, TXRIS behaviour
      5     => hse,      //= High-Speed enable
      7     => lbe,      //= UART loopback enable
      8     => txe,      //= UART TX enable
      9     => rxe,      //= UART RX enable
      11    => rts,      //= Request-to-Send
      14    => rtsen,    //= Enable Request-to-Send
      15    => ctsen,    //= Enable Clear-to-Send
    }
  });

  pub const UART_0: *const Uart = 0x4000C000 as *const Uart;
  pub const UART_1: *const Uart = 0x4000D000 as *const Uart;
  pub const UART_2: *const Uart = 0x4000E000 as *const Uart;
  pub const UART_3: *const Uart = 0x4000F000 as *const Uart;
  pub const UART_4: *const Uart = 0x40010000 as *const Uart;
  pub const UART_5: *const Uart = 0x40011000 as *const Uart;
  pub const UART_6: *const Uart = 0x40012000 as *const Uart;
  pub const UART_7: *const Uart = 0x40013000 as *const Uart;
}
