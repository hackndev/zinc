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

/*!
Universal synchronous asynchronous receiver transmitter (USART).

*/

use core::fmt;
use core::result::Result;
use core::intrinsics::abort;

use drivers::chario::CharIO;
use hal::uart;
use hal::stm32l1::init;

use self::UsartPeripheral::*;

#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Available USART peripherals.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Copy)]
pub enum UsartPeripheral {
  Usart1,
  Usart2,
  Usart3,
  Uart4,
  Uart5,
}

/// USART word length.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Copy)]
pub enum WordLen {
  WordLen8bits = 0,
  WordLen9bits = 1,
}

/// Stop bits configuration.
#[repr(u8)]
#[derive(Copy)]
pub enum StopBit {
  /// Single stop bit.
  StopBit1bit       = 0,
  /// A half stop bit.
  StopBit05bits    = 1,
  /// Two stop bits.
  StopBit2bits      = 2,
  /// One and a half stop bits.
  StopBit15bits    = 3,
}

/// Structure describing a USART instance.
#[derive(Copy)]
pub struct Usart {
  reg: &'static reg::USART,
}

impl Usart {
  /// Create a new USART port.
  pub fn new(peripheral: UsartPeripheral, baudrate: u32, word_len: WordLen,
             parity: uart::Parity, stop_bits: StopBit,
             config: &init::ClockConfig) -> Usart {
    use hal::stm32l1::peripheral_clock::PeripheralClock;
    use hal::stm32l1::peripheral_clock as clock;
    use hal::uart::Parity::*;

    let (reg, clock) = match peripheral {
        Usart1 => (&reg::USART1, PeripheralClock::Apb2(clock::BusApb2::Usart1)),
        Usart2 => (&reg::USART2, PeripheralClock::Apb1(clock::BusApb1::Usart2)),
        Usart3 => (&reg::USART3, PeripheralClock::Apb1(clock::BusApb1::Usart3)),
        Uart4  => (&reg::UART4,  PeripheralClock::Apb1(clock::BusApb1::Uart4)),
        Uart5  => (&reg::UART5,  PeripheralClock::Apb1(clock::BusApb1::Uart5)),
    };

    clock.enable();

    reg.cr1.set_word_length(word_len as usize != 0);
    reg.cr2.set_stop_bits(stop_bits as u16);

    // Standard USART baud rate:
    // Tx/Rx baud = Fck / (8 * (2 - OVER8) * USARTDIV)

    let bus_clock = clock.frequency(config);
    let over8 = reg.cr1.oversample_8bit_enable() as usize;
    let idiv = (bus_clock << 4) / (baudrate << (2 - over8));
    reg.brr.set_fraction(((idiv & 0xF) >> over8) as u16);
    reg.brr.set_mantissa((idiv >> 4) as u16);

    let (pe_on, pe_select) = match parity {
        Disabled => (false, false),
        Even => (true, false),
        Odd => (true, true),
        _ => unsafe { abort() }, // not supported
    };
    reg.cr1.set_parity_control_enable(pe_on);
    reg.cr1.set_parity_selection(pe_select);
    //reg.cr3.set_rts_enable(true);
    //reg.cr3.set_cts_enable(true);
    reg.cr1.set_transmitter_enable(true);
    reg.cr1.set_receiver_enable(true);

    reg.cr1.set_usart_enable(true);

    Usart {
      reg: reg,
    }
  }
}

impl CharIO for Usart {
  fn putc(&self, value: char) {
    wait_for!(self.reg.sr.transmit_data_empty());
    self.reg.dr.set_data(value as u16);
  }
}

impl fmt::Write for Usart {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    use core::str::StrExt;
    for b in s.bytes() {
      wait_for!(self.reg.sr.transmit_data_empty());
      self.reg.dr.set_data(b as u16);
    }
    Result::Ok(())
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(USART = {
    0x00 => reg16 sr {  // status
      0 => error_parity     : ro,
      1 => error_framing    : ro,
      2 => error_noise      : ro,
      3 => error_overrun    : ro,
      4 => idle_line        : ro,
      5 => read_data_not_empty      : ro,
      6 => transmission_complete    : ro,
      7 => transmit_data_empty      : ro,
      8 => lin_break        : ro,
      9 => cts              : ro
    },
    0x04 => reg16 dr {  // data
      8..0 => data : rw,
    },
    0x08 => reg16 brr { // baud rate
      3..0  => fraction : rw,
      15..4 => mantissa : rw,
    },
    0x0C => reg16 cr1 { // control 1
      0 => send_back : rw,
      1 => receiver_wakeup : rw,
      2 => receiver_enable : rw,
      3 => transmitter_enable : rw,
      4 => int_idle_enable : rw,
      5 => int_read_data_not_empty_enable : rw,
      6 => int_transmission_complete_enable : rw,
      7 => int_transmission_data_empty_enable : rw,
      8 => int_pe_enable : rw, // = USART_CR1_PEIE, not sure about it
      9 => parity_selection : rw,
      10 => parity_control_enable : rw,
      11 => wakeup_method : rw,
      12 => word_length : rw,
      13 => usart_enable : rw,
      // 14 => reserved : ro,
      15 => oversample_8bit_enable : rw,
    },
    0x10 => reg16 cr2 { // control 2
      3..0 => address : rw,
      5 => line_break_length : rw,
      6 => int_line_break_enable : rw,
      8 => last_bit_clock_pulse : rw,
      9 => clock_phase : rw,
      10 => clock_polarity : rw,
      11 => clock_enable : rw,
      13..12 => stop_bits : rw,
      14 => line_mode_enable : rw,
    },
    0x14 => reg16 cr3 { // control 3
      0 => int_error_enable : rw,
      1 => irda_mode_enable : rw,
      2 => irda_low_power : rw,
      3 => half_duplex_selection : rw,
      4 => smartcard_nack_enable : rw,
      5 => smartcard_mode_enable : rw,
      6 => dma_receiver_enable : rw,
      7 => dma_transmitter_enable : rw,
      8 => rts_enable : rw,
      9 => cts_enable : rw,
      10 => int_cts_enable : rw,
      11 => one_sample_method_enable : rw,
    },
    0x18 => reg16 gtpr {    // guard time and prescaler
      7..0  => prescaler  : rw,
      15..8 => guard_time : rw,
    },
  });

  extern {
    #[link_name="stm32l1_iomem_USART1"] pub static USART1: USART;
    #[link_name="stm32l1_iomem_USART2"] pub static USART2: USART;
    #[link_name="stm32l1_iomem_USART3"] pub static USART3: USART;
    #[link_name="stm32l1_iomem_UART4"]  pub static UART4:  USART;
    #[link_name="stm32l1_iomem_UART5"]  pub static UART5:  USART;
  }
}
