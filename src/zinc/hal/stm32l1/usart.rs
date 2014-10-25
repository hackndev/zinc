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

use core::intrinsics::abort;

//use hal::lpc17xx::peripheral_clock::{PeripheralClock, UART0Clock, UART2Clock, UART3Clock};
use drivers::chario::CharIO;
use hal::uart;

#[path="../../util/ioreg.rs"] mod ioreg;
#[path="../../util/wait_for.rs"] mod wait_for;

/// Available USART peripherals.
#[allow(missing_doc)]
pub enum UARTPeripheral {
  USART1,
  USART2,
  USART3,
  UART4,
  UART5,
}

/// USART word length.
#[allow(missing_doc)]
pub enum WordLen {
  WordLen8bits = 0,
  WordLen9bits = 1,
}

impl WordLen {
  /// Convert a number into a WordLen.
  pub fn from_u8(val: u8) -> WordLen {
    match val {
      8 => WordLen8bits,
      9 => WordLen9bits,
      _ => unsafe { abort() },
    }
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
      15..0 => control : rw, //TODO
    },
    0x10 => reg16 cr2 { // control 2
      15..0 => control : rw, //TODO
    },
    0x14 => reg16 cr3 { // control 3
      15..0 => control : rw, //TODO
    },
    0x18 => reg16 gtpr {    // guard time and prescaler
      7..0  => prescaler  : rw,
      15..8 => guard_time : rw,
    },
  })

  extern {
    #[link_name="stm32l1_iomem_USART1"] pub static USART1: USART;
    #[link_name="stm32l1_iomem_USART2"] pub static USART2: USART;
    #[link_name="stm32l1_iomem_USART3"] pub static USART3: USART;
    #[link_name="stm32l1_iomem_UART4"]  pub static UART4:  USART;
    #[link_name="stm32l1_iomem_UART5"]  pub static UART5:  USART;
  }
}
