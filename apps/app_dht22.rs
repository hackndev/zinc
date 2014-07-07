#![crate_type="rlib"]
#![no_std]

extern crate zinc;
extern crate core;

use core::option::{Some, None};

use zinc::boards::mbed_lpc1768;
use zinc::drivers::chario::CharIO;
use zinc::hal::timer::{TimerConf, Timer};
use zinc::hal::uart::{UARTConf, Disabled};
use zinc::hal::pin::map;
use zinc::hal::gpio::{GPIOConf, Out};
use zinc::os::debug;
use zinc::drivers::dht22;

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx::init::SysConf;
#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;

struct Platform {
  configuration: SysConf,
  timer: TimerConf,
  uart: UARTConf,
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
};

#[no_split_stack]
pub fn main() {
  platform.configuration.setup();
  debug::setup(&platform.uart);

  let d = debug::io();
  d.puts("DHT22 demo\n");

  let gpio = &GPIOConf {
    pin: map::port0::pin4::GPIO,
    direction: Out,
  };
  let timer = &platform.timer.setup();

  let dht = dht22::DHT22::new(gpio, timer);

  loop {
    (timer as &Timer).wait(3);

    let ret = dht.read();
    match ret {
      Some(v) => {
        d.puts("temp:     "); d.puti(v.temperature as u32); d.puts("\n");
        d.puts("humidity: "); d.puti(v.humidity as u32); d.puts("\n");
      },
      None => {
        d.puts("fail\n");
      },
    }
  }
}
