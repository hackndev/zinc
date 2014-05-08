#![crate_id="app"]
#![crate_type="rlib"]
#![no_std]

extern crate zinc;
extern crate std;

use std::intrinsics::volatile_load;

#[cfg(mcu_lpc17xx)] use zinc::boards::mbed_lpc1768;
use zinc::hal::gpio::GPIOConf;
use zinc::hal::init::SysConf;
use zinc::hal::cortex_m3::systick;

struct Platform {
  configuration: SysConf,
  led1: GPIOConf,
}

#[cfg(mcu_lpc17xx)]
static platform: Platform = Platform {
  configuration: mbed_lpc1768::configuration,
  led1: mbed_lpc1768::led1,
};

static mut i: u32 = 0;
static mut on: u32 = 0;

/// TODO(farcaller): https://github.com/mozilla/rust/pull/13833
#[inline(always)]
#[no_split_stack]
#[link_section=".text.insignificant"]
unsafe fn systick_handler() {
  i += 1;
  if i > 100 {
    i = 0;
    on = !on;
  }
}

#[no_split_stack]
pub fn main() {
  platform.configuration.setup();
  systick::setup(systick::CALIBRATED, true);

  let led1 = platform.led1.setup();

  led1.set_high();

  let mut ison: bool = true;

  systick::enable();

  unsafe { loop {
    let on: bool = volatile_load(&on as *u32) == 0;
    if ison != on {
      ison = on;
      match ison {
        true  => led1.set_high(),
        false => led1.set_low(),
      }
    }
  } }
}
