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
Peripheral clock management.

This module should be considered private until further notice.
*/

use core::intrinsics::abort;

use super::system_clock::system_clock;
use self::PeripheralClock::*;
use self::PeripheralDivisor::*;
use core::marker::Copy;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;

/// Configures the state of peripheral clock.
#[allow(missing_docs)]
pub enum PeripheralClock {
  // reserved = 0,
  TIM0Clock  = 1,
  TIM1Clock  = 2,
  UART0Clock = 3,
  UART1Clock = 4,
  // reserved = 5,
  PWM1Clock  = 6,
  I2C0Clock  = 7,
  SPIClock   = 8,
  RTCClock   = 9,
  SSP1Clock  = 10,
  // reserved = 11,
  ADCClock   = 12,
  CAN1Clock  = 13,
  CAN2Clock  = 14,
  GPIOClock  = 15,
  RITClock   = 16,
  MCPWMClock = 17,
  QEIClock   = 18,
  I2C1Clock  = 19,
  // reserved = 20,
  SSP0Clock  = 21,
  TIM2Clock  = 22,
  TIM3Clock  = 23,
  UART2Clock = 24,
  UART3Clock = 25,
  I2C2Clock  = 26,
  I2SClock   = 27,
  // reserved = 28,
  GPDMAClock = 29,
  ENETClock  = 30,
  USBClock   = 31,
}

impl Copy for PeripheralClock {}

/// Configures the divisor of peripheral clock based on core clock.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum PeripheralDivisor {
  WDTDivisor     = 0,
  TIMER0Divisor  = 2,
  TIMER1Divisor  = 4,
  UART0Divisor   = 6,
  UART1Divisor   = 8,
  // reserved    = 10,
  PWM1Divisor    = 12,
  I2C0Divisor    = 14,
  SPIDivisor     = 16,
  // reserved    = 18,
  SSP1Divisor    = 20,
  DACDivisor     = 22,
  ADCDivisor     = 24,
  CAN1Divisor    = 26,
  CAN2Divisor    = 28,
  ACFDivisor     = 30,

  QEIDivisor     = 32,
  GPIOINTDivisor = 34,
  PCBDivisor     = 36,
  I2C1Divisor    = 38,
  // reserved    = 40,
  SSP0Divisor    = 42,
  TIMER2Divisor  = 44,
  TIMER3Divisor  = 46,
  UART2Divisor   = 48,
  UART3Divisor   = 50,
  I2C2Divisor    = 52,
  I2SDivisor     = 54,
  // reserved    = 56,
  RITDivisor     = 58,
  SYSCONDivisor  = 60,
  MCDivisor      = 62,
}

impl PeripheralClock {
  /// Enables the given peripheral clock.
  pub fn enable(self) {
    let bit: u32 = (1u32 << (self as usize)) as u32;
    let val: u32 = reg::PCONP.value();
    reg::PCONP.set_value(val | bit);
  }

  /// Disables the given peripheral clock.
  pub fn disable(self) {
    let bit: u32 = !((1u32 << (self as usize)) as u32);
    let val: u32 = reg::PCONP.value();
    reg::PCONP.set_value(val & bit);
  }

  /// Returns the clock frequency based on active divisor.
  pub fn frequency(self) -> u32 {
    system_clock() / self.get_divisor() as u32
  }

  /// Returns the given peripheral clock divisor.
  pub fn get_divisor(self) -> u8 {
    let (reg, offset) = self.divisor_reg_and_offset();
    let val = (reg.value() >> (offset as usize)) & 3;
    match val {
      1 => 1,
      2 => 2,
      0 => 4,
      3 => match self {
        CAN1Clock|CAN2Clock => 6,  // TODO(farcaller): wtf is CAN filter?
        _ => 8,
      },
      _ => unsafe { abort() },
    }
  }

  /// Sets the given peripheral clock divisor.
  pub fn set_divisor(self, divisor: u8) {
    self.verify_divisor(divisor);
    let (reg, offset) = self.divisor_reg_and_offset();
    let divisor_value: u8 = match divisor {
      1   => 1,
      2   => 2,
      4   => 0,
      8|6 => 3,
      _   => unsafe { abort() },
    };

    let bits: u32 = (divisor_value as u32) << (offset as usize);
    let mask: u32 = !((3u32 << (offset as usize)) as u32);
    let val: u32 = reg.value();
    reg.set_value(val & mask | bits);
  }

  fn verify_divisor(self, divisor: u8) {
    match divisor {
      1|2|4|8 => (),
      6 => match self {
        CAN1Clock|CAN2Clock => (),  // TODO(farcaller): wtf is CAN filter?
        _ => unsafe { abort() },
      },
      _ => unsafe { abort() },
    }
  }

  fn to_divisor(self) -> PeripheralDivisor {
    match self {
      TIM0Clock  => TIMER0Divisor,
      TIM1Clock  => TIMER1Divisor,
      UART0Clock => UART0Divisor,
      UART1Clock => UART1Divisor,
      PWM1Clock  => PWM1Divisor,
      I2C0Clock  => I2C0Divisor,
      SPIClock   => SPIDivisor,
      RTCClock   => unsafe { abort() },
      SSP1Clock  => SSP1Divisor,
      ADCClock   => ADCDivisor,
      CAN1Clock  => CAN1Divisor,
      CAN2Clock  => CAN2Divisor,
      GPIOClock  => unsafe { abort() },
      RITClock   => RITDivisor,
      MCPWMClock => MCDivisor,
      QEIClock   => QEIDivisor,
      I2C1Clock  => I2C1Divisor,
      SSP0Clock  => SSP0Divisor,
      TIM2Clock  => TIMER2Divisor,
      TIM3Clock  => TIMER3Divisor,
      UART2Clock => UART2Divisor,
      UART3Clock => UART3Divisor,
      I2C2Clock  => I2C2Divisor,
      I2SClock   => I2SDivisor,
      GPDMAClock => unsafe { abort() },
      ENETClock  => unsafe { abort() },
      USBClock   => unsafe { abort() },
    }
  }

  fn divisor_reg_and_offset(self) -> (&'static reg::PCLKSEL, u32) {
    match self.to_divisor() {
      WDTDivisor|TIMER0Divisor|TIMER1Divisor|UART0Divisor|UART1Divisor|
      PWM1Divisor|I2C0Divisor|SPIDivisor|SSP1Divisor|DACDivisor|ADCDivisor|
      CAN1Divisor|CAN2Divisor|ACFDivisor => (&reg::PCLKSEL0, self as u32),

      QEIDivisor|GPIOINTDivisor|PCBDivisor|I2C1Divisor|SSP0Divisor|
      TIMER2Divisor|TIMER3Divisor|UART2Divisor|UART3Divisor|I2C2Divisor|
      I2SDivisor|RITDivisor|SYSCONDivisor|
      MCDivisor => (&reg::PCLKSEL1, self as u32 - 32),
    }
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;

  ioreg_old!(PCONP: u32, value);
  reg_rw!(PCONP, u32, value, set_value, value);

  ioreg_old!(PCLKSEL: u32, value);
  reg_rw!(PCLKSEL, u32, value, set_value, value);

  extern {
    #[link_name="lpc17xx_iomem_PCONP"] pub static PCONP: PCONP;
    #[link_name="lpc17xx_iomem_PCLKSEL0"] pub static PCLKSEL0: PCLKSEL;
    #[link_name="lpc17xx_iomem_PCLKSEL1"] pub static PCLKSEL1: PCLKSEL;
  }
}
