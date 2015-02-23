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

use core::option::Option;
use core::marker::Copy;

use super::sim;

use self::Port::*;
use self::Function::*;
use self::PullConf::*;
use self::DriveStrength::*;
use self::SlewRate::*;

/// A pin.
#[allow(missing_docs)]
#[derive(Copy)]
pub struct Pin {
  pub port: Port,
  pub pin: u8,
}

/// Available port names.
#[allow(missing_docs)]
pub enum Port {
  PortA = 1,
  PortB = 2,
  PortC = 3,
  PortD = 4,
  PortE = 5,
}

impl Copy for Port {}

/// Pin functions (GPIO or up to seven additional functions).
#[derive(PartialEq)]
#[allow(missing_docs)]
pub enum Function {
  Analog       = 0,
  Gpio         = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
  AltFunction4 = 4,
  AltFunction5 = 5,
  AltFunction6 = 6,
  AltFunction7 = 7,
}

impl Copy for Function {}

/// Pull-up/-down configuration.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum PullConf {
  PullNone   = 0,
  PullUp     = 1,
  PullDown   = 2,
}

/// Pin output driver strength.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum DriveStrength {
  DriveStrengthHigh   = 0,
  DriveStrengthLow    = 1,
}

/// Pin output drive slew rate.
#[allow(missing_docs)]
#[derive(Copy)]
pub enum SlewRate {
  SlewFast   = 0,
  SlewSlow   = 1,
}

impl Pin {
  /// Create and setup a Pin.
  pub fn new(port: Port, pin_index: u8, function: Function,
      gpiodir: Option<::hal::pin::GpioDirection>) -> Pin {
    let pin = Pin {
      port: port,
      pin: pin_index,
    };
    pin.setup_regs(function, gpiodir, PullNone,
                   DriveStrengthHigh, SlewSlow, false, false);

    pin
  }

  fn setup_regs(&self, function: Function,
      gpiodir: Option<::hal::pin::GpioDirection>,
      pull: PullConf, drive_strength: DriveStrength,
      slew_rate: SlewRate, filter: bool, open_drain: bool) {
    use self::reg::Port_pcr_ps as ps;
    use self::reg::Port_pcr_sre as sre;
    use self::reg::Port_pcr_dse as dse;

    // enable port clock
    sim::enable_PORT(self.port);

    let (pe, ps) = match pull {
      PullNone => (false, ps::PULL_DOWN),
      PullDown => (true,  ps::PULL_DOWN),
      PullUp   => (true,  ps::PULL_UP),
    };
    let sre = match slew_rate {
      SlewFast => sre::FAST,
      SlewSlow => sre::SLOW,
    };
    let dse = match drive_strength {
      DriveStrengthHigh => dse::HIGH_DRIVE,
      DriveStrengthLow  => dse::LOW_DRIVE,
    };

    self.pcr()
      .set_pe(pe)
      .set_ps(ps)
      .set_sre(sre)
      .set_pfe(filter)
      .set_ode(open_drain)
      .set_dse(dse)
      .set_mux(function as u32);

    if function == Gpio {
      (self as &::hal::pin::Gpio).set_direction(gpiodir.unwrap());
    }
  }

  fn gpioreg(&self) -> &'static reg::Gpio {
    match self.port {
      PortA => &reg::GPIO_A,
      PortB => &reg::GPIO_B,
      PortC => &reg::GPIO_C,
      PortD => &reg::GPIO_D,
      PortE => &reg::GPIO_E,
    }
  }

  fn pcr(&self) -> &'static reg::Port_pcr {
    let port: &reg::Port = match self.port {
      PortA => &reg::PORT_A,
      PortB => &reg::PORT_B,
      PortC => &reg::PORT_C,
      PortD => &reg::PORT_D,
      PortE => &reg::PORT_E,
    };
    return &port.pcr[self.pin as usize];
  }
}

impl ::hal::pin::Gpio for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpioreg().psor.set_ptso(self.pin as usize, true);
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpioreg().pcor.set_ptco(self.pin as usize, true);
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GpioLevel {
    let reg = self.gpioreg();
    match reg.pdir.pdi(self.pin as usize) {
      false => ::hal::pin::Low,
      _     => ::hal::pin::High,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, new_mode: ::hal::pin::GpioDirection) {
    use self::reg::Gpio_pddr_pdd as pdd;
    let reg = self.gpioreg();
    let val = match new_mode {
      ::hal::pin::In  => pdd::INPUT,
      ::hal::pin::Out => pdd::OUTPUT,
    };
    reg.pddr.set_pdd(self.pin as usize, val);
  }
}

/// Register definitions
pub mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(Port = {
    /// Port control register
    0x0    => reg32 pcr[32]
    {
      0      => ps {  //= Pull direction select
        0 => PULL_DOWN,
        1 => PULL_UP
      }
      1      => pe,   //= Pull enable
      2      => sre { //= Slew rate
        0 => FAST,
        1 => SLOW
      }
      4      => pfe,  //= Passive filter enable
      5      => ode,  //= Open drain enable
      6      => dse { //= Drive strength
        0 => LOW_DRIVE,
        1 => HIGH_DRIVE
      }
      8..10  => mux,  //= Multiplexer configuration
      15     => lk,   //= Configuration lock
      16..19 => irqc {//= Interrupt configuration
        0  => IRQ_NONE,        //= No IRQ enablled
        1  => IRQ_DMA_RISING,  //= Trigger DMA on rising edge
        2  => IRQ_DMA_FALLING, //= Trigger DMA on falling edge
        3  => IRQ_DMA_EITHER,  //= Trigger DMA on either edge
        // reserved
        8  => IRQ_ZERO,
        9  => IRQ_RISING,
        10 => IRQ_FALLING,
        11 => IRQ_EITHER,
        12 => IRQ_ONE,
      }
    }

    0x80   => reg32 gpclr {   //= Global pin control low
      0..15  => gpwd,
      16..31 => gpwe,
    }

    0x84   => reg32 gpchr {   //= Global pin control high
      0..15  => gpwd,
      16..31 => gpwe,
    }

    0x88   => reg32 isfr {    //= Interrupt status
      0..31  => isf
    }
  });

  extern {
    #[link_name="k20_iomem_PORTA"] pub static PORT_A: Port;
    #[link_name="k20_iomem_PORTB"] pub static PORT_B: Port;
    #[link_name="k20_iomem_PORTC"] pub static PORT_C: Port;
    #[link_name="k20_iomem_PORTD"] pub static PORT_D: Port;
    #[link_name="k20_iomem_PORTE"] pub static PORT_E: Port;
  }

  ioregs!(Gpio = {
    0x0     => reg32 pdo {  //! port data output register
      0..31   => pdo
    }

    0x4     => reg32 psor { //! port set output register
      0..31   => ptso[32]: wo
    }

    0x8     => reg32 pcor { //! port clear output register
      0..31   => ptco[32]: wo
    }

    0xc     => reg32 ptor { //! port toggle output register
      0..31   => ptto[32]: wo
    }

    0x10    => reg32 pdir { //! port data input register
      0..31   => pdi[32]
    }

    0x14    => reg32 pddr { //! port direction register
      0..31   => pdd[32] {
        0 => INPUT,
        1 => OUTPUT,
      }
    }
  });

  extern {
    #[link_name="k20_iomem_GPIOA"] pub static GPIO_A: Gpio;
    #[link_name="k20_iomem_GPIOB"] pub static GPIO_B: Gpio;
    #[link_name="k20_iomem_GPIOC"] pub static GPIO_C: Gpio;
    #[link_name="k20_iomem_GPIOD"] pub static GPIO_D: Gpio;
    #[link_name="k20_iomem_GPIOE"] pub static GPIO_E: Gpio;
  }
}
