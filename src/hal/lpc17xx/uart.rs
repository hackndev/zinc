// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
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

This code doesn't support UART1, while it really should (UART1 has more features
than other UARTs in MCU).
*/

use core::intrinsics::abort;

use hal::lpc17xx::peripheral_clock::PeripheralClock;
use hal::lpc17xx::peripheral_clock::PeripheralClock::UART0Clock;
use hal::lpc17xx::peripheral_clock::PeripheralClock::UART2Clock;
use hal::lpc17xx::peripheral_clock::PeripheralClock::UART3Clock;
use drivers::chario::CharIO;
use hal::uart;

use self::UARTPeripheral::*;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;


/// Available UART peripherals.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum UARTPeripheral {
  UART0,
  UART2,
  UART3,
}

/// UART word length.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum WordLen {
  WordLen5bits = 0b00,
  WordLen6bits = 0b01,
  WordLen7bits = 0b10,
  WordLen8bits = 0b11,
}

impl WordLen {
  /// Convert a number into a WordLen.
  pub fn from_u8(val: u8) -> WordLen {
    use self::WordLen::*;
    match val {
      5 => WordLen5bits,
      6 => WordLen6bits,
      7 => WordLen7bits,
      8 => WordLen8bits,
      _ => unsafe { abort() },
    }
  }
}

/// Stop bits configuration.
#[derive(Clone, Copy)]
pub enum StopBit {
  /// Single stop bit.
  StopBit1bit  = 0b0_00,
  /// Two stop bits.
  StopBit2bits = 0b1_00,
}

impl StopBit {
  /// Convert a number into a StopBit.
  pub fn from_u8(val: u8) -> StopBit {
    use self::StopBit::*;
    match val {
      1 => StopBit1bit,
      2 => StopBit2bits,
      _ => unsafe { abort() },
    }
  }
}

enum ParityEnabled {
  PEDisabled = 0b0_0_00,
  PEEnabled  = 0b1_0_00,
}

enum ParitySelect {
  PSOdd    = 0b00_0_0_00,
  PSEven   = 0b01_0_0_00,
  PSForced1 = 0b10_0_0_00,
  PSForced0 = 0b11_0_0_00,
}

#[allow(dead_code)]
enum BreakControl {
  BCDisabled = 0b0_00_0_0_00,
  BCEnabled  = 0b1_00_0_0_00,
}

enum FIFOEnabled {
  FEEnabled  = 0b1,
  FEDisabled = 0b0,
}

#[allow(dead_code)]
enum FIFODmaMode {
  FDEnabled  = 0b1_0_0_0,
  FDDisabled = 0b0_0_0_0,
}

#[allow(dead_code)]
enum FIFOTriggerLevel {
  FT1char   = 0b00_00_0_0_0_0,
  FT4chars  = 0b01_00_0_0_0_0,
  FT8chars  = 0b10_00_0_0_0_0,
  FT14chars = 0b11_00_0_0_0_0,
}

/// Structure describing a UART instance.
#[derive(Clone)]
pub struct UART {
  reg: &'static reg::UART,
  clock: PeripheralClock,
}

impl UARTPeripheral {
  fn reg(self) -> &'static reg::UART {
    match self {
      UART0 => &reg::UART0,
      UART2 => &reg::UART2,
      UART3 => &reg::UART3,
    }
  }

  fn peripheral_clock(self) -> PeripheralClock {
    match self {
      UART0 => UART0Clock,
      UART2 => UART2Clock,
      UART3 => UART3Clock,
    }
  }
}

impl UART {
  /// Create ans setup a UART.
  pub fn new(peripheral: UARTPeripheral, baudrate: u32, word_len: u8,
      parity: uart::Parity, stop_bits: u8) -> UART {
    let uart = UART {
      reg: peripheral.reg(),
      clock: peripheral.peripheral_clock(),
    };

    uart.clock.enable();
    uart.set_baud_rate(baudrate);
    uart.set_mode(WordLen::from_u8(word_len), parity,
        StopBit::from_u8(stop_bits));
    uart.set_fifo_enabled(true, true);

    uart
  }

  fn uart_clock(&self) -> u32 {
    self.clock.frequency()
  }

  fn set_baud_rate(&self, baud_rate: u32) {
    self.reg.set_LCR(0b1000_0000); // enable divisor latch access

    let (dl, div_add_val, mul_val) = self.calculate_divisors(baud_rate);

    self.reg.set_DLM((dl >> 8) & 0xff);
    self.reg.set_DLL(dl & 0xff);
    self.reg.set_FDR(div_add_val | (mul_val << 4));

    self.reg.set_LCR(3);
  }

  fn set_mode(&self, word_len: WordLen, parity: uart::Parity, stop_bits: StopBit) {
    use self::ParityEnabled::*;
    use self::ParitySelect::*;
    let lcr: u8 = (*(self.reg)).LCR() as u8;
    let computed_val: u8 = word_len as u8 | stop_bits as u8 | match parity {
      uart::Parity::Disabled => PEDisabled as u8  | PSOdd as u8,
      uart::Parity::Odd      => PEEnabled as u8   | PSOdd as u8,
      uart::Parity::Even     => PEEnabled as u8   | PSEven as u8,
      uart::Parity::Forced1  => PEEnabled as u8   | PSForced1 as u8,
      uart::Parity::Forced0  => PEEnabled as u8   | PSForced0 as u8,
    };
    let new_lcr = (lcr & !LCRModeMask) | computed_val;

    (*(self.reg)).set_LCR(new_lcr as u32);
  }

  fn set_fifo_enabled(&self, enabled: bool, reset: bool) {
    use self::FIFOEnabled::*;
    use self::FIFODmaMode::*;
    use self::FIFOTriggerLevel::*;
    let val: u8 = match enabled {
      true => FEEnabled as u8,
      false => FEDisabled as u8
    } | match reset {
      true  => FIFOResetTx & FIFOResetRx,
      false => 0,
    } | FDDisabled as u8 | FT1char as u8;

    (*(self.reg)).set_FCR(val as u32);
  }

  // TODO(farcaller): license note
  // loosely based on serial_api.c
  // Copyright (c) 2006-2013 ARM Limited
  // Licensed under the Apache License, Version 2.0
  fn calculate_divisors(&self, baud_rate: u32) -> (u32, u32, u32) {
    let baudrate: u32 = baud_rate;
    let pclk = self.uart_clock();
    let mut dl: u32 = pclk / (16 * baudrate);

    let mut div_add_val: u32 = 0;
    let mut mul_val: u32 = 1;
    let mut hit = false;
    let mut dlv: u32;
    let mut mv: u32;
    let mut dav: u32;
    if pclk % (16 * baudrate) != 0 {     // Checking for zero remainder
      let mut err_best: u32 = baudrate;
      let mut b: i32;

      mv = 1;
      while mv < 16 && !hit {
        dav = 0;
        while dav < mv {
          if (mv * pclk * 2) & 0x80000000 == 0x80000000 {
            dlv = ((((2 * mv * pclk) / (baudrate * (dav + mv))) / 16) + 1) / 2;
          } else {
            dlv = ((((4 * mv * pclk) / (baudrate * (dav + mv))) / 32) + 1) / 2;
          }

          if dlv == 0 {
            dlv = 1;
          }

          if (dav > 0) && (dlv < 2) {
            dlv = 2;
          }

          b = (((pclk * mv / (dlv * (dav + mv) * 8)) + 1) / 2) as i32;

          b = b - baudrate as i32;
          if b < 0 {
            b = -b;
          }
          if (b as u32) < err_best {
            err_best  = b as u32;

            dl        = dlv;
            mul_val    = mv;
            div_add_val = dav;

            if (b as u32) == baudrate {
              hit = true;
              break;
            }
          }
          dav += 1;
        }
        mv += 1;
      }
    }
    return (dl, div_add_val, mul_val)
  }
}

impl CharIO for UART {
  fn putc(&self, value: char) {
    wait_for!(self.reg.LSR() as u8 & LSRTHREmpty == LSRTHREmpty);
    self.reg.set_THR(value as u32);
  }
}

#[allow(non_upper_case_globals)]
static FIFOResetRx: u8 = 0b1_0;
#[allow(non_upper_case_globals)]
static FIFOResetTx: u8 = 0b1_0_0;

#[allow(non_upper_case_globals)]
static LCRModeMask: u8 = 0b1_11_1_1_11;

#[allow(non_upper_case_globals)]
static LSRTHREmpty: u8 = 0x20;

mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(UART: u32, RBR_THR_DLL, DLM_IER, IIR_FCR, LCR, _pad_0, LSR, _pad_1, SCR, ACR, ICR, FDR, _pad_2, TER);
  reg_r!( UART, u32, RBR,          RBR_THR_DLL);
  reg_w!( UART, u32,      set_THR, RBR_THR_DLL);
  reg_rw!(UART, u32, DLL, set_DLL, RBR_THR_DLL);
  reg_rw!(UART, u32, DLM, set_DLM, DLM_IER);
  reg_rw!(UART, u32, IER, set_IER, DLM_IER);
  reg_r!( UART, u32, IIR,          IIR_FCR);
  reg_w!( UART, u32,      set_FCR, IIR_FCR);
  reg_rw!(UART, u32, LCR, set_LCR, LCR);
  reg_r!( UART, u32, LSR,          LSR);
  reg_rw!(UART, u32, SCR, set_SCR, SCR);
  reg_rw!(UART, u32, ACR, set_ACR, ACR);
  reg_rw!(UART, u32, ICR, set_ICR, ICR);
  reg_rw!(UART, u32, FDR, set_FDR, FDR);
  reg_rw!(UART, u32, TER, set_TER, TER);

  extern {
    #[link_name="lpc17xx_iomem_UART0"] pub static UART0: UART;
    #[link_name="lpc17xx_iomem_UART2"] pub static UART2: UART;
    #[link_name="lpc17xx_iomem_UART3"] pub static UART3: UART;
  }
}
