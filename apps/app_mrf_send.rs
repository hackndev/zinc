#![crate_id="app"]
#![crate_type="rlib"]
#![no_std]

extern crate zinc;
extern crate std;

use std::str::{Str, StrSlice};
use std::slice::Vector;

use zinc::boards::mbed_lpc1768;
use zinc::drivers::chario::CharIO;
use zinc::drivers::mrf24j40;
use zinc::hal::gpio::{GPIOConf, In, Out};
use zinc::hal::pin::{map, Connected};
use zinc::hal::spi::SPIConf;
use zinc::hal::timer::{TimerConf, Timer};
use zinc::hal::uart::{UARTConf, Disabled};

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx::init::SysConf;
#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;

struct Platform {
  configuration: SysConf,
  timer: TimerConf,
  uart: UARTConf,
  spi: SPIConf,
  cs: GPIOConf,
  reset: GPIOConf,
  interrupt: GPIOConf,
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
  uart: UARTConf {
    peripheral: lpc17xx::uart::UART0,
    baudrate: 115200,
    word_len: 8,
    parity: Disabled,
    stop_bits: 1,

    tx: map::port0::pin2::TXD0,
    rx: map::port0::pin3::RXD0,
  },
  spi: SPIConf {
    peripheral: lpc17xx::ssp::SSP1,
    bits: 8,
    mode: 0,
    frequency: 20_000000,
    mosi: Connected(map::port0::pin9::MOSI1),
    miso: Connected(map::port0::pin8::MISO1),
    sclk: Connected(map::port0::pin7::SCK1),
  },
  cs: GPIOConf {
    pin: map::port0::pin6::GPIO,
    direction: Out,
  },
  reset: GPIOConf {
    pin: map::port0::pin1::GPIO,
    direction: Out,
  },
  interrupt: GPIOConf {
    pin: map::port2::pin5::GPIO,
    direction: In,
  }
};

#[no_split_stack]
pub fn main() {
  platform.configuration.setup();

  let uart = &platform.uart.setup() as &CharIO;
  let spi = platform.spi.setup();
  let timer = platform.timer.setup();

  uart.puts("MRF init...\n");

  let mrf = mrf24j40::Mrf24j40::new(&spi, &timer, &platform.reset, &platform.cs,
      &platform.interrupt, 12);

  mrf.set_pan(0xcafe);
  mrf.set_short_address(0x6001);

  uart.puts("PAN: "); uart.puth(mrf.get_pan() as u32); uart.puts("\n");
  uart.puts("adr: "); uart.puth(mrf.get_short_address() as u32); uart.puts("\n");
  loop {
    uart.puts("Sending...\n");
    mrf.send_to_short_address(0x6002, "hello".as_slice().as_bytes());
    timer.wait(5);
  }
}
