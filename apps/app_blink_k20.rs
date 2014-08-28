#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

use core::option::Some;
use zinc::hal::k20::{pin, watchdog};
use zinc::hal::pin::GPIO;
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

#[no_mangle]
#[no_split_stack]
#[allow(unused_variable)]
#[allow(dead_code)]
pub unsafe fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();
  watchdog::init(watchdog::Disabled);

  // Pins for MC HCK (http://www.mchck.org/)
  let led1 = pin::Pin::new(pin::PortB, 16, pin::GPIO, Some(zinc::hal::pin::Out));

  systick::setup(480000, false);
  systick::enable();
  loop {
    led1.set_high();
    wait(10);
    led1.set_low();
    wait(10);
  }
}
