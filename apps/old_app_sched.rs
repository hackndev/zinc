#![crate_type="rlib"]
#![no_std]
#![feature(globs, macro_rules, asm)]

extern crate zinc;

#[cfg(mcu_lpc17xx)] use zinc::boards::mbed_lpc1768;
use zinc::hal::gpio::GPIOConf;
use zinc::hal::init::SysConf;
use zinc::hal::timer::{TimerConf, Timer};
use zinc::hal::uart::{UARTConf, Disabled};
use zinc::hal::pin::map;
use zinc::drivers::chario::CharIO;
use zinc::os::task;
use zinc::os::debug;

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;

struct Platform {
  configuration: SysConf,
  led1: GPIOConf,
  led2: GPIOConf,
  timer: TimerConf,
  uart: UARTConf,
}

#[cfg(mcu_lpc17xx)]
static platform: Platform = Platform {
  configuration: mbed_lpc1768::configuration,
  led1: mbed_lpc1768::led1,
  led2: mbed_lpc1768::led4,
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

fn task(arg: u32) {
  let timer = &platform.timer.setup() as &Timer;
  timer.wait(1);

  let d = debug::io();
  d.puts("task "); d.puti(arg); d.puts(" started\n");

  loop {
    timer.wait(2);
    d.puts("running in task "); d.puti(arg); d.puts("\n");
  }
}

fn main_task(arg: u32) {
  let timer = &platform.timer.setup() as &Timer;
  let d = debug::io();
  d.puts("task "); d.puti(arg); d.puts(" started\n");

  task::define_task(task, 1, 512, false);
  task::define_task(task, 2, 512, false);

  loop {
    timer.wait(2);
    d.puts("running in main task\n");
  }
}

pub fn main() {
  platform.configuration.setup();
  debug::setup(&platform.uart);

  task::setup(main_task, 512);
}
