#![feature(no_std, core, start)]
#![no_std]

extern crate core;
extern crate zinc;

use core::option::Option::Some;
use zinc::hal::k20::{pin, watchdog};
use zinc::hal::pin::Gpio;
use zinc::hal::cortex_m4::systick;

/// Wait the given number of SysTick ticks
pub fn wait(ticks: u32) {
  let mut n = ticks;
  // Reset the tick flag
  systick::tick();
  loop {
    if systick::tick() {
      n -= 1;
      if n == 0 {
        break;
      }
    }
  }
}

pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();
  watchdog::init(watchdog::State::Disabled);

  // Pins for MC HCK (http://www.mchck.org/)
  let led1 = pin::Pin::new(pin::Port::PortB, 16, pin::Function::Gpio, Some(zinc::hal::pin::Out));

  systick::setup(systick::ten_ms().unwrap_or(480000));
  systick::enable();
  loop {
    led1.set_high();
    wait(10);
    led1.set_low();
    wait(10);
  }
}

#[start]
fn start(_: isize, _: *const *const u8) -> isize {
    main();
    0
}
