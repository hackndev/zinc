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
UART configuration.
*/

use core::intrinsics::abort;

use drivers::chario::CharIO;
use hal::uart;

#[path="../../lib/ioreg.rs"] mod ioreg;
#[path="../../lib/wait_for.rs"] mod wait_for;

/// Available UART peripherals.
#[allow(missing_doc)]
pub enum UARTPeripheral {
  UART0,
  UART1,
  UART2,
}

/// Structure describing a UART instance.
pub struct UART {
  reg: &'static reg::UART,
}

/// UART word length.
#[allow(missing_doc)]
pub enum WordLen {
  WordLen8bits = 0,
  WordLen9bits = 0b00010000,
}

impl WordLen {
  /// Convert from number to WordLen.
  pub fn from_u8(val: u8) -> WordLen {
    match val {
      8 => WordLen8bits,
      9 => WordLen9bits,
      _ => unsafe { abort() },
    }
  }
}

/// Stop bits configuration.
/// K20 UART only supports one stop bit.
pub enum StopBit {
  /// Single stop bit.
  StopBit1bit  = 0,
}

impl StopBit {
  /// Convert from number to StopBit.
  pub fn from_u8(val: u8) -> StopBit {
    match val {
      1 => StopBit1bit,
      _ => unsafe { abort() },
    }
  }
}

enum ParityEnabled {
  PEDisabled = 0b00,
  PEEnabled  = 0b10,
}

enum ParitySelect {
  PSOdd    = 0b1,
  PSEven   = 0b0,
}

impl UARTPeripheral {
  fn reg(self) -> &'static reg::UART {
    match self {
      UART0 => &reg::UART0,
      UART1 => &reg::UART1,
      UART2 => &reg::UART2,
    }
  }
}

impl UART {
  /// Returns platform-specific UART object that implements CharIO trait.
  pub fn new(peripheral: UARTPeripheral, baudrate:  u32, word_len: u8,
      parity: uart::Parity, stop_bits: u8) -> UART {
    let uart = UART {
      reg: peripheral.reg()
    };
    uart.set_baud_rate(baudrate);
    uart.set_mode(WordLen::from_u8(word_len), parity,
        StopBit::from_u8(stop_bits));
    uart.set_fifo_enabled(true);

    uart
  }

  #[no_split_stack]
  fn uart_clock(&self) -> u32 {
    48000000 // FIXME(bgamari): Use peripheral clocks
  }

  #[no_split_stack]
  fn set_baud_rate(&self, baud_rate: u32) {
    let sbr: u32 = self.uart_clock() / 16 / baud_rate;
    let brfa: u32 = (2 * self.uart_clock() / baud_rate) % 32;
    (*self.reg).set_BDH((sbr >> 8) as u8);
    (*self.reg).set_BDL((sbr & 0xff) as u8);
    (*self.reg).set_C4(((*self.reg).C4() & !0b11111) | brfa as u8);
  }

  #[no_split_stack]
  fn set_mode(&self, word_len: WordLen, parity: uart::Parity, _: StopBit) {
    let c1: u8 = (*(self.reg)).C1();
    let computed_val: u8 = word_len as u8 | match parity {
      uart::Disabled => PEDisabled as u8  | PSOdd as u8,
      uart::Odd      => PEEnabled as u8   | PSOdd as u8,
      uart::Even     => PEEnabled as u8   | PSEven as u8,
      uart::Forced1  => unsafe { abort() },
      uart::Forced0  => unsafe { abort() },
    };
    let new_c1 = (c1 & !0x3) | computed_val;

    (*(self.reg)).set_C1(new_c1);
  }

  fn set_fifo_enabled(&self, enabled: bool) {
    let val: u8 = match enabled {
      true => PFIFOTxFifoEnabled | PFIFORxFifoEnabled,
      false => 0,
    };

    (*(self.reg)).set_PFIFO(val);
  }
}

impl CharIO for UART {
  fn putc(&self, value: char) {
    wait_for!(self.reg.S1() as u8 & S1TDREmpty == S1TDREmpty);
    self.reg.set_D(value as u8);
  }
}

static S1TDREmpty: u8 = 0b1000_0000;

static PFIFOTxFifoEnabled: u8 = 0b1000_0000;
static PFIFORxFifoEnabled: u8 = 0b0000_1000;

mod reg {
  use lib::volatile_cell::VolatileCell;

  ioreg_old!(UART: u8, BDH, BDL, C1, C2, S1, S2, C3, D, MA1, MA2, C4, C5, ED, MODEM, IR,
         PFIFO, CFIFO, SFIFO, TWFIFO, TCFIFO, RWFIFO, RCFIFO)
  reg_rw!(UART, u8, BDH,     set_BDH,     BDH)
  reg_rw!(UART, u8, BDL,     set_BDL,     BDL)
  reg_rw!(UART, u8, C1,      set_C1,      C1)
  reg_rw!(UART, u8, C2,      set_C2,      C2)
  reg_r!( UART, u8, S1,                   S1)
  reg_rw!(UART, u8, S2,      set_S2,      S2)
  reg_rw!(UART, u8, C3,      set_C3,      C3)
  reg_rw!(UART, u8, D,       set_D,       D)
  reg_rw!(UART, u8, MA1,     set_MA1,     MA1)
  reg_rw!(UART, u8, MA2,     set_MA2,     MA2)
  reg_rw!(UART, u8, C4,      set_C4,      C4)
  reg_rw!(UART, u8, C5,      set_C5,      C5)
  reg_rw!(UART, u8, ED,      set_ED,      ED)
  reg_rw!(UART, u8, MODEM,   set_MODEM,   MODEM)
  reg_rw!(UART, u8, IR,      set_IR,      IR)
  reg_rw!(UART, u8, PFIFO,   set_PFIFO,   PFIFO)
  reg_rw!(UART, u8, CFIFO,   set_CFIFO,   CFIFO)
  reg_rw!(UART, u8, SFIFO,   set_SFIFO,   SFIFO)
  reg_rw!(UART, u8, TWFIFO,  set_TWFIFO,  TWFIFO)
  reg_r!( UART, u8, TCFIFO,               TCFIFO)
  reg_rw!(UART, u8, RWFIFO,  set_RWFIFO,  RWFIFO)
  reg_r!( UART, u8, RCFIFO,               RCFIFO)
  // FIXME(bgamari): Specialized registers omitted

  extern {
    #[link_name="k20_iomem_UART1"] pub static UART0: UART;
    #[link_name="k20_iomem_UART2"] pub static UART1: UART;
    #[link_name="k20_iomem_UART3"] pub static UART2: UART;
  }
}
