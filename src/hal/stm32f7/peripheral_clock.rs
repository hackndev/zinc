// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
// Adapted from stm32f4/peripheral_clock.rs for stm32f7 by Dave Hylands <dhylands@gmail.com>
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

//! Peripheral clock management routines.
//!
//! This module should be considered private until further notice.
//!
//! Note: this module is used as part of initial setup if PLL is used.

use super::init::reg;
use core::marker::Copy;

use self::PeripheralClock::*;

#[path="../../util/ioreg.rs"] mod ioreg;

/// Configures the state of peripheral clock.
///
/// This enum contains all available clocks from both AHB and APB.
#[allow(missing_docs)]
#[derive(Clone)]
pub enum PeripheralClock {
  // AHB1
  GPIOAClock,
  GPIOBClock,
  GPIOCClock,
  GPIODClock,
  GPIOEClock,
  GPIOFClock,
  GPIOGClock,
  GPIOHClock,
  GPIOIClock,
  GPIOJClock,
  GPIOKClock,
  CRCClock,
  BKPSRAMClock,
  DTCMClock,
  DMA1Clock,
  DMA2Clock,
  DMA2DClock,
  ETHMACClock,
  ETHMACTxClock,
  ETHMACRxClock,
  ETHMACPTPClock,
  OTGHSClock,
  OTGHSULPIClock,

  // AHB2
  DCMIClock,
  CRYPClock,
  HASHClock,
  RNGClock,
  OTGFSClock,

  // AHB3
  FSMCClock,
  QSPIClock,

  // APB1
  TIM2Clock,
  TIM3Clock,
  TIM4Clock,
  TIM5Clock,
  TIM6Clock,
  TIM7Clock,
  TIM12Clock,
  TIM13Clock,
  TIM14Clock,
  LPTIM1Clock,
  WWDGClock,
  SPI2Clock,
  SPI3Clock,
  SPDIFClock,
  USART2Clock,
  USART3Clock,
  UART4Clock,
  UART5Clock,
  I2C1Clock,
  I2C2Clock,
  I2C3Clock,
  I2C4Clock,
  CAN1Clock,
  CAN2Clock,
  CECClock,
  PWRClock,
  DACClock,
  UART7Clock,
  UART8Clock,

  // APB2
  TIM1Clock,
  TIM8Clock,
  USART1Clock,
  USART6Clock,
  ADC1Clock,
  ADC2Clock,
  ADC3Clock,
  SDMMC1Clock,
  SPI1Clock,
  SYSCFGClock,
  TIM9Clock,
  TIM10Clock,
  TIM11Clock,
  SPI5Clock,
  SPI6Clock,
  SAI1Clock,
  SAI2Clock,
  LTDCClock,
}

impl Copy for PeripheralClock {}

impl PeripheralClock {
  /// Enables the given peripheral clock.
  pub fn enable(self) {
    self.set_reg(true);
  }
  /// Disables the given peripheral clock.
  pub fn disable(self) {
    self.set_reg(false);
  }

  fn to_reg_bit(self) -> u32 {
    1 << match self {
      GPIOAClock      => 0,
      GPIOBClock      => 1,
      GPIOCClock      => 2,
      GPIODClock      => 3,
      GPIOEClock      => 4,
      GPIOFClock      => 5,
      GPIOGClock      => 6,
      GPIOHClock      => 7,
      GPIOIClock      => 8,
      GPIOJClock      => 9,
      GPIOKClock      => 10,
      CRCClock        => 12,
      BKPSRAMClock    => 18,
      DTCMClock       => 20,
      DMA1Clock       => 21,
      DMA2Clock       => 22,
      DMA2DClock      => 23,
      ETHMACClock     => 25,
      ETHMACTxClock   => 26,
      ETHMACRxClock   => 27,
      ETHMACPTPClock  => 28,
      OTGHSClock      => 29,
      OTGHSULPIClock  => 30,

      DCMIClock       => 0,
      CRYPClock       => 4,
      HASHClock       => 5,
      RNGClock        => 6,
      OTGFSClock      => 7,

      FSMCClock       => 0,
      QSPIClock       => 1,

      TIM2Clock       => 0,
      TIM3Clock       => 1,
      TIM4Clock       => 2,
      TIM5Clock       => 3,
      TIM6Clock       => 4,
      TIM7Clock       => 5,
      TIM12Clock      => 6,
      TIM13Clock      => 7,
      TIM14Clock      => 8,
      LPTIM1Clock     => 9,
      WWDGClock       => 11,
      SPI2Clock       => 14,
      SPI3Clock       => 15,
      SPDIFClock      => 16,
      USART2Clock     => 17,
      USART3Clock     => 18,
      UART4Clock      => 19,
      UART5Clock      => 20,
      I2C1Clock       => 21,
      I2C2Clock       => 22,
      I2C3Clock       => 23,
      I2C4Clock       => 24,
      CAN1Clock       => 25,
      CAN2Clock       => 26,
      CECClock        => 27,
      PWRClock        => 28,
      DACClock        => 29,
      UART7Clock      => 30,
      UART8Clock      => 31,

      TIM1Clock       => 0,
      TIM8Clock       => 1,
      USART1Clock     => 4,
      USART6Clock     => 5,
      ADC1Clock       => 8,
      ADC2Clock       => 9,
      ADC3Clock       => 10,
      SDMMC1Clock     => 11,
      SPI1Clock       => 12,
      SYSCFGClock     => 14,
      TIM9Clock       => 16,
      TIM10Clock      => 17,
      TIM11Clock      => 18,
      SPI5Clock       => 20,
      SPI6Clock       => 21,
      SAI1Clock       => 22,
      SAI2Clock       => 23,
      LTDCClock       => 26,
    }
  }

  fn set_reg(self, enable: bool) {
    let reg_bit = self.to_reg_bit();
    let mask: u32 = !reg_bit;
    let bit: u32 = if enable {reg_bit} else {0};
    match self {
      GPIOAClock|GPIOBClock|GPIOCClock|GPIODClock|GPIOEClock|GPIOFClock|
      GPIOGClock|GPIOHClock|GPIOIClock|GPIOJClock|GPIOKClock|CRCClock|
      BKPSRAMClock|DTCMClock|DMA1Clock|DMA2Clock|DMA2DClock|
      ETHMACClock|ETHMACTxClock|ETHMACRxClock|
      ETHMACPTPClock|OTGHSClock|OTGHSULPIClock => {
        reg::RCC.ahb1enr.set_enable((reg::RCC.ahb1enr.enable() & mask) | bit);
      },
      DCMIClock|CRYPClock|HASHClock|RNGClock|OTGFSClock => {
        reg::RCC.ahb2enr.set_enable((reg::RCC.ahb2enr.enable() & mask) | bit);
      },
      FSMCClock|QSPIClock => {
        reg::RCC.ahb3enr.set_enable((reg::RCC.ahb3enr.enable() & mask) | bit);
      },
      TIM2Clock|TIM3Clock|TIM4Clock|TIM5Clock|TIM6Clock|TIM7Clock|TIM12Clock|
      TIM13Clock|TIM14Clock|LPTIM1Clock|WWDGClock|SPI2Clock|SPI3Clock|SPDIFClock|
      USART2Clock|USART3Clock|UART4Clock|UART5Clock|I2C1Clock|I2C2Clock|I2C3Clock|
      I2C4Clock|CAN1Clock|CAN2Clock|CECClock|PWRClock|DACClock|UART7Clock|UART8Clock => {
        reg::RCC.apb1enr.set_enable((reg::RCC.apb1enr.enable() & mask) | bit);
      },
      TIM1Clock|TIM8Clock|USART1Clock|USART6Clock|ADC1Clock|ADC2Clock|ADC3Clock|
      SDMMC1Clock|SPI1Clock|SYSCFGClock|TIM9Clock|TIM10Clock|TIM11Clock|
      SPI5Clock|SPI6Clock|SAI1Clock|SAI2Clock|LTDCClock => {
        reg::RCC.apb2enr.set_enable((reg::RCC.apb2enr.enable() & mask) | bit);
      },
    }
  }
}
