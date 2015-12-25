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
Pin configuration.

Some pins that could be configured here may be missing from actual MCU depending
on the package.
*/

use core::intrinsics::abort;
use core::option::Option;

use self::Port::*;

#[path="../../util/ioreg.rs"]
#[macro_use] mod ioreg;
#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

/// Available port names.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Port {
  Port0,
  Port1,
  Port2,
  Port3,
  Port4,
}

/// Pin functions (GPIO or up to three additional functions).
#[derive(PartialEq, Clone, Copy)]
#[allow(missing_docs)]
pub enum Function {
  Gpio         = 0,
  AltFunction1 = 1,
  AltFunction2 = 2,
  AltFunction3 = 3,
}

/// Pin modes
#[derive(PartialEq, Clone, Copy)]
#[allow(missing_docs)]
pub enum Mode {
  PullUp = 0,
  Repeater = 1,
  Floating = 2,
  PullDown = 3,
}

/// Structure to describe the location of a pin
#[derive(Clone, Copy)]
pub struct Pin {
  /// Port the pin is attached to
  port: Port,
  /// Pin number in the port
  pin: u8
}

impl Pin {
  /// Create and setup a Pin
  pub fn new(port: Port, pin_index: u8, function: Function,
      gpiodir: Option<::hal::pin::GpioDirection>) -> Pin {
    let pin = Pin {
      port: port,
      pin: pin_index,
    };

    pin.setup_regs(function, gpiodir);

    pin
  }

  fn setup_regs(&self, function: Function,
      gpiodir: Option<::hal::pin::GpioDirection>) {
    let (offset, reg) = self.get_pinsel_reg_and_offset();

    let fun_bits: u32  = (function as u32) << ((offset as usize) * 2);
    let mask_bits: u32 = !(3u32 << ((offset as usize) * 2));

    let val: u32 = reg.value();
    let new_val = (val & mask_bits) | fun_bits;
    reg.set_value(new_val);

    match function {
      Function::Gpio => (self as &::hal::pin::Gpio).set_direction(gpiodir.unwrap()),
      Function::AltFunction1 => match self.adc_channel() {
          Some(_) => self.setup_adc(),
          _ => {},
        },
      _ => {},
    }
  }

  fn set_mode(&self, mode: Mode) {
    let (offset, reg) = self.get_pimode_reg_and_offset();
    let value = reg.value() | (mode as u32) << offset;
    reg.set_value(value)
  }

  fn gpioreg(&self) -> &reg::Gpio {
    match self.port {
      Port0 => &reg::GPIO_0,
      Port1 => &reg::GPIO_1,
      Port2 => &reg::GPIO_2,
      Port3 => &reg::GPIO_3,
      Port4 => &reg::GPIO_4,
    }
  }

  fn get_pinsel_reg_and_offset(&self) -> (u8, &reg::PINSEL) {
    match self.port {
      Port0 => match self.pin {
        0...15  => (self.pin,    &reg::PINSEL0),
        16...30 => (self.pin-16, &reg::PINSEL1),
        _      => unsafe { abort() },
      },
      Port1 => match self.pin {
        0...15  => (self.pin,    &reg::PINSEL2),
        16...31 => (self.pin-16, &reg::PINSEL3),
        _      => unsafe { abort() },
      },
      Port2 => match self.pin {
        0...13  => (self.pin,    &reg::PINSEL4),
        _      => unsafe { abort() },
      },
      Port3 => match self.pin {
        25|26 => (self.pin-16,  &reg::PINSEL7),
        _     => unsafe { abort() },
      },
      Port4 => match self.pin {
        28|29 => (self.pin-16,  &reg::PINSEL9),
        _     => unsafe { abort() },
      },
    }
  }

  fn get_pimode_reg_and_offset(&self) -> (u8, &reg::PINMODE) {
    match self.port {
      Port0 => match self.pin {
        0...11  => (self.pin*2, &reg::PINMODE0),
        15      => (self.pin*2, &reg::PINMODE0),
        16...26 => ((self.pin-16)*2, &reg::PINMODE1),
        _       => unsafe { abort() },
      },
      Port1 => match self.pin {
        0...1   => (self.pin*2, &reg::PINMODE2),
        4       => (self.pin*2, &reg::PINMODE2),
        8...10  => (self.pin*2, &reg::PINMODE2),
        14...15 => (self.pin*2, &reg::PINMODE2),
        16...31 => ((self.pin-16)*2, &reg::PINMODE3),
        _      => unsafe { abort() },
      },
      Port2 => match self.pin {
        0...13 => (self.pin*2, &reg::PINMODE4),
        _      => unsafe { abort() },
      },
      Port3 => match self.pin {
        25 => (18, &reg::PINMODE7),
        26 => (20, &reg::PINMODE7),
        _  => unsafe { abort() },
      },
      Port4 => match self.pin {
        28 => (24, &reg::PINMODE9),
        29 => (26, &reg::PINMODE9),
        _  => unsafe { abort() },
      },
    }
  }

  /// Get adc channel number
  fn adc_channel(&self) -> Option<u8> {
    match self.port {
      Port0 => match self.pin { 
        2 => Some(7),
        3 => Some(6),
        23...26 => Some(self.pin - 23),
        _ => None,
      },
      Port1 => match self.pin {
        30...31 => Some(self.pin - 26),
        _ => None,
      },
      Port2 => None,
      Port3 => None,
      Port4 => None,
    }
  }

  fn setup_adc(&self) {
    // ensure power is turned on
    let pconp = &reg::PCONP;
    let pconp_value = pconp.value();
    pconp.set_value(pconp_value | (1 << 12));

    // set PCLK of ADC to /1
    let pclksel0 = &reg::PCLKSEL0;
    let mut pclksel0_val: u32 = pclksel0.value();
    pclksel0_val &= !(0x3 << 24);
    pclksel0_val |= 0x1 << 24;
    pclksel0.set_value(pclksel0_val);

    fn div_round_up(x: u32, y: u32) -> u32 {
      (x + (y - 1)) / y
    }
    let pclk = ::hal::lpc17xx::system_clock::system_clock();
    let max_adc_clk = 13000000;
    let clkdiv = div_round_up(pclk, max_adc_clk);

    let cr = (0 << 0)      // SEL: 0 = no channels selected
           | (clkdiv << 8) // CLKDIV: PCLK max ~= 25MHz, /25 to give safe 1MHz at fastest
           | (0 << 16)     // BURST: 0 = software control
           | (0 << 17)     // CLKS: not applicable
           | (1 << 21)     // PDN: 1 = operational
           | (0 << 24)     // START: 0 = no start
           | (0 << 27);    // EDGE: not applicable
    &reg::ADC.set_CR(cr);

    self.set_mode(Mode::Floating);
  }
}

impl ::hal::pin::Gpio for Pin {
  /// Sets output GPIO value to high.
  fn set_high(&self) {
    self.gpioreg().set_FIOSET(1 << (self.pin as usize));
  }

  /// Sets output GPIO value to low.
  fn set_low(&self) {
    self.gpioreg().set_FIOCLR(1 << (self.pin as usize));
  }

  /// Returns input GPIO level.
  fn level(&self) -> ::hal::pin::GpioLevel {
    let bit: u32 = 1 << (self.pin as usize);
    let reg = self.gpioreg();

    match reg.FIOPIN() & bit {
      0 => ::hal::pin::Low,
      _ => ::hal::pin::High,
    }
  }

  /// Sets output GPIO direction.
  fn set_direction(&self, new_mode: ::hal::pin::GpioDirection) {
    let bit: u32 = 1 << (self.pin as usize);
    let mask: u32 = !bit;
    let reg = self.gpioreg();
    let val: u32 = reg.FIODIR();
    let new_val: u32 = match new_mode {
      ::hal::pin::In  => val & mask,
      ::hal::pin::Out => (val & mask) | bit,
    };

    reg.set_FIODIR(new_val);
  }

}

impl ::hal::pin::Adc for Pin {
  /// Read analog input value of pin
  fn read(&self) -> u32 {
    let adc = &reg::ADC;
    let channel = self.adc_channel().unwrap();
    let mut cr = adc.CR();
    cr &= !(0xFF as u32);
    cr |= 1 << channel;
    cr |= (1 << 24) as u32;
    adc.set_CR(cr);

    wait_for!((adc.STAT() & (1 << channel)) != 0);

    let data = match channel {
      0 => adc.DR0(),
      1 => adc.DR1(),
      2 => adc.DR2(),
      3 => adc.DR3(),
      4 => adc.DR4(),
      5 => adc.DR5(),
      6 => adc.DR6(),
      7 => adc.DR7(),
      _ => unsafe { abort() },
    };

    adc.set_CR((adc.CR() as u32) & !(1 << 24));
    (data >> 4) & 0xFFF // 12 bit range
  }
}

/// Sets the state of trace port interface.
pub fn set_trace_port_interface_enabled(enabled: bool) {
  let value: u32 = if enabled { 0b1000 } else { 0 };
  reg::PINSEL10.set_value(value);
}

mod reg {
  use volatile_cell::VolatileCell;

  ioreg_old!(PINSEL: u32, value);
  reg_rw!(PINSEL, u32, value, set_value, value);

  extern {
    #[link_name="lpc17xx_iomem_PINSEL0"]  pub static PINSEL0:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL1"]  pub static PINSEL1:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL2"]  pub static PINSEL2:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL3"]  pub static PINSEL3:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL4"]  pub static PINSEL4:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL7"]  pub static PINSEL7:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL9"]  pub static PINSEL9:  PINSEL;
    #[link_name="lpc17xx_iomem_PINSEL10"] pub static PINSEL10: PINSEL;
  }

  ioreg_old!(PINMODE: u32, value);
  reg_rw!(PINMODE, u32, value, set_value, value);
  extern {
    #[link_name="lpc17xx_iomem_PINMODE0"] pub static PINMODE0: PINMODE;
    #[link_name="lpc17xx_iomem_PINMODE1"] pub static PINMODE1: PINMODE;
    #[link_name="lpc17xx_iomem_PINMODE2"] pub static PINMODE2: PINMODE;
    #[link_name="lpc17xx_iomem_PINMODE3"] pub static PINMODE3: PINMODE;
    #[link_name="lpc17xx_iomem_PINMODE4"] pub static PINMODE4: PINMODE;
    #[link_name="lpc17xx_iomem_PINMODE7"] pub static PINMODE7: PINMODE;
    #[link_name="lpc17xx_iomem_PINMODE9"] pub static PINMODE9: PINMODE;
  }

  ioreg_old!(Gpio: u32, FIODIR, _r0, _r1, _r2, FIOMASK, FIOPIN, FIOSET, FIOCLR);
  reg_rw!(Gpio, u32, FIODIR,  set_FIODIR,  FIODIR);
  reg_rw!(Gpio, u32, FIOMASK, set_FIOMASK, FIOMASK);
  reg_rw!(Gpio, u32, FIOPIN,  set_FIOPIN,  FIOPIN);
  reg_rw!(Gpio, u32, FIOSET,  set_FIOSET,  FIOSET);
  reg_rw!(Gpio, u32, FIOCLR,  set_FIOCLR,  FIOCLR);

  extern {
    #[link_name="lpc17xx_iomem_GPIO0"] pub static GPIO_0: Gpio;
    #[link_name="lpc17xx_iomem_GPIO1"] pub static GPIO_1: Gpio;
    #[link_name="lpc17xx_iomem_GPIO2"] pub static GPIO_2: Gpio;
    #[link_name="lpc17xx_iomem_GPIO3"] pub static GPIO_3: Gpio;
    #[link_name="lpc17xx_iomem_GPIO4"] pub static GPIO_4: Gpio;
  }


  ioreg_old!(PCONP: u32, value);
  ioreg_old!(PCLKSEL0: u32, value);
  reg_rw!(PCONP, u32, value, set_value, value);
  reg_rw!(PCLKSEL0, u32, value, set_value, value);
  extern {
    #[link_name="lpc17xx_iomem_PCONP"]  pub static PCONP:  PCONP;
    #[link_name="lpc17xx_iomem_PCLKSEL0"]  pub static PCLKSEL0: PCLKSEL0;
  }

  ioreg_old!(ADC: u32, CR, GDR, pad_0, INTEN, DR0, DR1, DR2, DR3, DR4, DR5, DR6, DR7, STAT, TRM);
  reg_rw!(ADC, u32, CR, set_CR, CR);
  reg_r!(ADC, u32, GDR, GDR);
  reg_rw!(ADC, u32, INTEN, set_INTEN, INTEN);
  reg_rw!(ADC, u32, DR0, set_DR0, DR0);
  reg_rw!(ADC, u32, DR1, set_DR0, DR1);
  reg_rw!(ADC, u32, DR2, set_DR0, DR2);
  reg_rw!(ADC, u32, DR3, set_DR0, DR3);
  reg_rw!(ADC, u32, DR4, set_DR0, DR4);
  reg_rw!(ADC, u32, DR5, set_DR0, DR5);
  reg_rw!(ADC, u32, DR6, set_DR0, DR6);
  reg_rw!(ADC, u32, DR7, set_DR0, DR7);
  reg_rw!(ADC, u32, STAT, set_STAT, STAT);

  extern {
    #[link_name="lpc17xx_iomem_ADC"]  pub static ADC:  ADC;
  }
}
