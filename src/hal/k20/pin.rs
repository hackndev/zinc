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
Pin configuration.

Some pins that could be configured here may be missing from actual MCU depending
on the package.
*/

use core::option::{Option};

use lib::volatile_cell::VolatileCell;

use super::sim;

#[path="../../lib/ioreg.rs"] mod ioreg;

/// A pin
pub struct Pin {
  pub port: Port,
  pub pin: u8,
}

/// Available port names.
pub enum Port {
  PortA = 1,
  PortB = 2,
  PortC = 3,
  PortD = 4,
  PortE = 5,
}

/// Pin functions (GPIO or up to seven additional functions).
#[deriving(PartialEq)]
pub enum Function {
  Analog       = 0,
  GPIO         = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
  AltFunction4 = 4,
  AltFunction5 = 5,
  AltFunction6 = 6,
  AltFunction7 = 7,
}

/// Pull-up/-down configuration
pub enum PullConf {
  PullNone   = 0,
  PullUp     = 1,
  PullDown   = 2,
}

/// Pin output driver strength
pub enum DriveStrength {
  DriveStrengthHigh   = 0,
  DriveStrengthLow    = 1,
}

/// Pin output drive slew rate
pub enum SlewRate {
  SlewFast   = 0,
  SlewSlow   = 1,
}

impl Pin {
  pub fn new(port: Port, pin_index: u8, function: Function,
      gpiodir: Option<::hal::pin::GPIODirection>) -> Pin {
    let pin = Pin {
      port: port,
      pin: pin_index,
    };
    pin.setup_regs(function, gpiodir, PullNone,
                   DriveStrengthHigh, SlewSlow, false, false);

    pin
  }

  pub fn setup_regs(&self, function: Function,
      gpiodir: Option<::hal::pin::GPIODirection>,
      pull: PullConf, drive_strength: DriveStrength,
      slew_rate: SlewRate, filter: bool, open_drain: bool) {
    // enable port clock
    match self.port {
      PortA => sim::reg::SIM.scgc5.set_porta(true),
      PortB => sim::reg::SIM.scgc5.set_portb(true),
      PortC => sim::reg::SIM.scgc5.set_portc(true),
      PortD => sim::reg::SIM.scgc5.set_portd(true),
      PortE => sim::reg::SIM.scgc5.set_porte(true),
    }

    let value =
          (pull as u32 << 0)
          | (slew_rate as u32 << 2)
          | (filter as u32 << 4)
          | (open_drain as u32 << 5)
          | (drive_strength as u32 << 6)
          | (function as u32 << 8);
    //self.pcr().set(value); // FIXME

    if function == GPIO {
      (self as &::hal::pin::GPIO).set_direction(gpiodir.unwrap());
    }
  }

  fn gpioreg(&self) -> &reg::GPIO {
    match self.port {
      PortA => &reg::GPIOA,
      PortB => &reg::GPIOB,
      PortC => &reg::GPIOC,
      PortD => &reg::GPIOD,
      PortE => &reg::GPIOE,
    }
  }

  fn pcr(&self) -> &reg::PORT_pcr {
    let port: &reg::PORT = match self.port {
      PortA => &reg::PORTA,
      PortB => &reg::PORTB,
      PortC => &reg::PORTC,
      PortD => &reg::PORTD,
      PortE => &reg::PORTE,
    };
    return &port.pcr[self.pin as uint];
  }
}

impl ::hal::pin::GPIO for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpioreg().psor.set_ptso(self.pin as uint, true);
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpioreg().pcor.set_ptco(self.pin as uint, true);
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GPIOLevel {
    let reg = self.gpioreg();
    match reg.pdir.pdi(self.pin as uint) {
      false => ::hal::pin::Low,
      _     => ::hal::pin::High,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, new_mode: ::hal::pin::GPIODirection) {
    let reg = self.gpioreg();
    let val = match new_mode {
      ::hal::pin::In  => reg::INPUT,
      ::hal::pin::Out => reg::OUTPUT,
    };
    reg.pddr.set_pdd(self.pin as uint, val);
  }
}

mod reg {
  ioregs!(PORT = {

    0x0    => reg32 pcr[32]     /// Port control register
    {
      0      => ps    /// Pull direction select
        { 0 => PULL_DOWN,
          1 => PULL_UP
        }
      1      => pe,   /// Pull enable
      2      => sre   /// Slew rate
        { 0 => FAST,
          1 => SLOW
        }
      4      => pfe,  /// Passive filter enable
      5      => ode,  /// Open drain enable
      6      => dse   /// Drive strength
        { 0 => LOW_DRIVE,
          1 => HIGH_DRIVE
        }
      8..10  => mux,  /// Multiplexer configuration
      15     => lk,   /// Configuration lock
      16..19 => irqc  /// Interrupt configuration
        { 0  => IRQ_NONE,
          1  => IRQ_DMA_RISING,
          2  => IRQ_DMA_FALLING,
          3  => IRQ_DMA_EITHER,
          // reserved
          8  => IRQ_ZERO,
          9  => IRQ_RISING,
          10 => IRQ_FALLING,
          11 => IRQ_EITHER,
          12 => IRQ_ONE,
        }
    }

    0x80   => reg32 gpclr     /// Global pin control low
    {
      0..15  => gpwd,
      16..31 => gpwe,
    }

    0x84   => reg32 gpchr     /// Global pin control high
    {
      0..15  => gpwd,
      16..31 => gpwe,
    }

    0x88   => reg32 isfr      /// Interrupt status
    {0..31  => isf}
  })

  extern {
    #[link_name="iomem_PORTA"] pub static PORTA: PORT;
    #[link_name="iomem_PORTB"] pub static PORTB: PORT;
    #[link_name="iomem_PORTC"] pub static PORTC: PORT;
    #[link_name="iomem_PORTD"] pub static PORTD: PORT;
    #[link_name="iomem_PORTE"] pub static PORTE: PORT;
  }

  ioregs!(GPIO = {
    0x0     => reg32 pdo  /// port data output register
      {0..31   => pdo}

    0x4     => reg32 psor /// port set output register
      {0..31   => ptso[32]}

    0x8     => reg32 pcor /// port clear output register
      {0..31   => ptco[32]}

    0xc     => reg32 ptor /// port toggle output register
      {0..31   => ptto[32]}

    0x10    => reg32 pdir /// port data input register
      {0..31   => pdi[32]}

    0x14    => reg32 pddr /// port direction register
      {
        0..31   => pdd[32] {
          0 => INPUT,
          1 => OUTPUT,
        }
      }
  })

  extern {
    #[link_name="iomem_GPIOA"] pub static GPIOA: GPIO;
    #[link_name="iomem_GPIOB"] pub static GPIOB: GPIO;
    #[link_name="iomem_GPIOC"] pub static GPIOC: GPIO;
    #[link_name="iomem_GPIOD"] pub static GPIOD: GPIO;
    #[link_name="iomem_GPIOE"] pub static GPIOE: GPIO;
  }
}
