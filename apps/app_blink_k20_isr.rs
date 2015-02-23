#![feature(plugin, no_std, core)]
#![feature(asm)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;

use core::intrinsics::volatile_load;

use core::option::Option::Some;
use zinc::hal::k20::{pin, watchdog};
use zinc::hal::pin::Gpio;
use zinc::hal::cortex_m4::systick;
use zinc::util::support::wfi;

static mut i: u32 = 0;
static mut global_on: u32 = 0;

#[allow(dead_code)]
#[no_mangle]
pub unsafe extern fn isr_systick() {
    i += 1;
    if i > 100 {
      i = 0;
      global_on = !global_on;
    }
}

#[no_mangle]
#[allow(dead_code)]
pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();
  watchdog::init(watchdog::State::Disabled);

  // Pins for MC HCK (http://www.mchck.org/)
  let led1 = pin::Pin::new(pin::Port::PortB, 16, pin::Function::Gpio, Some(zinc::hal::pin::Out));

  systick::setup(systick::ten_ms().unwrap_or(480000));
  systick::enable();
  systick::enable_irq();

  loop {
    let on: bool = unsafe { volatile_load(&global_on as *const u32) == 0 };
      match on {
        true  => led1.set_high(),
        false => led1.set_low(),
      }
      wfi();
  }
}
