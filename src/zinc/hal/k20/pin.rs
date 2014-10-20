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

use super::sim;

/// A pin.
#[allow(missing_doc)]
pub struct Pin {
  pub port: Port,
  pub pin: u8,
}

/// Available port names.
#[allow(missing_doc)]
pub enum Port {
  PortA = 1,
  PortB = 2,
  PortC = 3,
  PortD = 4,
  PortE = 5,
}

/// Pin functions (GPIO or up to seven additional functions).
#[deriving(PartialEq)]
#[allow(missing_doc)]
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

/// Pull-up/-down configuration.
#[allow(missing_doc)]
pub enum PullConf {
  PullNone   = 0,
  PullUp     = 1,
  PullDown   = 2,
}

/// Pin output driver strength.
#[allow(missing_doc)]
pub enum DriveStrength {
  DriveStrengthHigh   = 0,
  DriveStrengthLow    = 1,
}

/// Pin output drive slew rate.
#[allow(missing_doc)]
pub enum SlewRate {
  SlewFast   = 0,
  SlewSlow   = 1,
}

impl Pin {
  /// Create and setup a Pin in open-drain mode.
  pub fn new(port: Port, pin_index: u8, function: Function,
      pull: PullConf, open_drain: bool) -> Pin {
    let pin = Pin {
      port: port,
      pin: pin_index,
    };
    pin.setup_regs(function, pull, DriveStrengthHigh, SlewSlow,
                   false, open_drain);

    pin
  }

  fn setup_regs(&self, function: Function,
      pull: PullConf, drive_strength: DriveStrength,
      slew_rate: SlewRate, filter: bool, open_drain: bool) {
    // enable port clock
    sim::enable_PORT(self.port);

    let (pe, ps) = match pull {
      PullNone => (false, reg::PULL_DOWN),
      PullDown => (true,  reg::PULL_DOWN),
      PullUp   => (true,  reg::PULL_UP),
    };
    let sre = match slew_rate {
      SlewFast => reg::FAST,
      SlewSlow => reg::SLOW,
    };
    let dse = match drive_strength {
      DriveStrengthHigh => reg::HIGH_DRIVE,
      DriveStrengthLow  => reg::LOW_DRIVE,
    };

    self.pcr()
      .set_pe(pe)
      .set_ps(ps)
      .set_sre(sre)
      .set_pfe(filter)
      .set_ode(open_drain)
      .set_dse(dse)
      .set_mux(function as u32);
  }

  fn pcr(&self) -> &'static reg::PORT_pcr {
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

/// A pin configured as a GPIO
pub struct GpioPin {
  pin: Pin
}

impl GpioPin {
  /// Configure a `Pin` as a GPIO pin.
  pub fn from_pin(pin: Pin, gpiodir: ::hal::pin::GPIODirection) -> GpioPin {
    let pin = GpioPin {pin: pin};
    (&pin as &::hal::pin::GPIO).set_direction(gpiodir);
    pin
  }

  /// Create and setup a GPIO Pin.
  pub fn new(port: Port, pin_index: u8,
      gpiodir: ::hal::pin::GPIODirection) -> GpioPin {
    GpioPin::from_pin(Pin::new(port, pin_index, GPIO, PullNone, false), gpiodir)
  }

  fn gpioreg(&self) -> &'static reg::GPIO {
    match self.pin.port {
      PortA => &reg::GPIOA,
      PortB => &reg::GPIOB,
      PortC => &reg::GPIOC,
      PortD => &reg::GPIOD,
      PortE => &reg::GPIOE,
    }
  }
}

impl ::hal::pin::GPIO for GpioPin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpioreg().psor.set_ptso(self.pin.pin as uint, true);
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpioreg().pcor.set_ptco(self.pin.pin as uint, true);
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GPIOLevel {
    let reg = self.gpioreg();
    match reg.pdir.pdi(self.pin.pin as uint) {
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
    reg.pddr.set_pdd(self.pin.pin as uint, val);
  }
}

/// Register definitions
pub mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(PORT = {
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
  })

  extern {
    #[link_name="k20_iomem_PORTA"] pub static PORTA: PORT;
    #[link_name="k20_iomem_PORTB"] pub static PORTB: PORT;
    #[link_name="k20_iomem_PORTC"] pub static PORTC: PORT;
    #[link_name="k20_iomem_PORTD"] pub static PORTD: PORT;
    #[link_name="k20_iomem_PORTE"] pub static PORTE: PORT;
  }

  ioregs!(GPIO = {
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
  })

  extern {
    #[link_name="k20_iomem_GPIOA"] pub static GPIOA: GPIO;
    #[link_name="k20_iomem_GPIOB"] pub static GPIOB: GPIO;
    #[link_name="k20_iomem_GPIOC"] pub static GPIOC: GPIO;
    #[link_name="k20_iomem_GPIOD"] pub static GPIOD: GPIO;
    #[link_name="k20_iomem_GPIOE"] pub static GPIOE: GPIO;
  }
}
