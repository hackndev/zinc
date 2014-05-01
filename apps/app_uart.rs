#![crate_id="app"]
#![crate_type="rlib"]
#![no_std]

extern crate zinc;

use zinc::boards::mbed_lpc1768;
use zinc::interfaces::chario::CharIO;
use zinc::hal::timer::{TimerConf, Timer};
use zinc::hal::uart::{UARTConf, None};
use zinc::hal::pin::map;
use zinc::hal::gpio::GPIOConf;

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx::init::SysConf;
#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;

struct Platform {
  configuration: SysConf,
  timer: TimerConf,
  uart: UARTConf,
  txled: GPIOConf,
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
    parity: None,
    stop_bits: 1,

    tx: map::port0::pin2::TXD0,
    rx: map::port0::pin3::RXD0,
  },
  txled: mbed_lpc1768::led4,
};

#[no_split_stack]
pub fn main() {
  platform.configuration.setup();

  let uart = &platform.uart.setup() as &CharIO;
  let timer = &platform.timer.setup() as &Timer;
  let txled = platform.txled.setup();

  uart.puts("Hello, world\n");

  let mut i = 0;
  loop {
    txled.set_high();
    uart.puts("Waiting for ");
    uart.puti(i);
    uart.puts(" seconds...\n");

    i += 1;
    txled.set_low();

    timer.wait(1);
  }
}
