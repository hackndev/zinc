#![crate_type="rlib"]
#![no_std]

extern crate zinc;

use zinc::boards::mbed_lpc1768;
use zinc::drivers::chario::CharIO;
use zinc::hal::timer::{TimerConf, Timer};
use zinc::hal::spi::SPIConf;
use zinc::hal::pin::{map, NotConnected, Connected};
use zinc::hal::gpio::{GPIOConf, Out};
use zinc::drivers::lcd;

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx::init::SysConf;
#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;

struct Platform {
  configuration: SysConf,
  timer: TimerConf,
  spi: SPIConf,
  lcd_dc: GPIOConf,
  lcd_cs: GPIOConf,
  lcd_reset: GPIOConf,
}

#[cfg(mcu_lpc17xx)]
#[address_insignificant]
static platform: Platform = Platform {
  configuration: mbed_lpc1768::configuration,
  timer: TimerConf {
    timer: lpc17xx::timer::Timer1,
    counter: 25,
    divisor: 4,
  },
  spi: SPIConf {
    peripheral: lpc17xx::ssp::SSP1,
    bits: 8,
    mode: 3,
    frequency: 6_000000,
    mosi: Connected(map::port0::pin9::MOSI1),
    miso: NotConnected,
    sclk: Connected(map::port0::pin7::SCK1),
  },
  lcd_dc: GPIOConf {
    pin: map::port0::pin6::GPIO,
    direction: Out,
  },
  lcd_cs: GPIOConf {
    pin: map::port0::pin18::GPIO,
    direction: Out,
  },
  lcd_reset: GPIOConf {
    pin: map::port0::pin8::GPIO,
    direction: Out,
  },
};

pub fn main() {
  platform.configuration.setup();

  let timer = platform.timer.setup();
  let spi = platform.spi.setup();
  let screen = &lcd::c12332::C12332::new(&spi, &timer, &platform.lcd_dc,
      &platform.lcd_cs, &platform.lcd_reset) as &lcd::LCD;

  screen.clear();
  screen.puts("hello lcd");
  screen.flush();
  let mut i = 0;
  loop {
    screen.clear();
    screen.puts("waiting ");
    screen.puti(i);
    screen.puts(" seconds");
    screen.flush();

    i += 1;
    timer.wait(1);
  }
}
