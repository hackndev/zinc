#![feature(phase)]
#![feature(asm)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

use core::intrinsics::volatile_load;

use zinc::hal::k20::watchdog;
use zinc::hal::pin::GPIO;
use zinc::hal::cortex_m4::systick;
use zinc::hal::board;

static mut i: u32 = 0;
static mut global_on: u32 = 0;

#[allow(dead_code)]
#[no_split_stack]
#[no_mangle]
pub unsafe extern fn isr_systick() {
    i += 1;
    if i > 100 {
      i = 0;
      global_on = !global_on;
    }
}

#[no_mangle]
#[no_split_stack]
#[allow(dead_code)]
pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();
  watchdog::init(watchdog::Disabled);

  let led1 = board::open_led();

  systick::setup(systick::ten_ms().unwrap_or(480000));
  systick::enable();
  systick::enable_irq();

  loop {
    let on: bool = unsafe { volatile_load(&global_on as *const u32) == 0 };
      match on {
        true  => led1.set_high(),
        false => led1.set_low(),
      }
    unsafe { asm!("wfi" :::: "volatile"); }
  }
}
