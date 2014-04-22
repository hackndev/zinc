#![feature(asm)]
#![crate_type="staticlib"]
#![no_std]

extern crate zinc;

// use zinc::hal;

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx::init::{SysConf, Clock, Main, PLL0};

struct Platform {
  configuration: SysConf,
}

#[cfg(mcu_lpc17xx)]
static platform: Platform = Platform {
  configuration: SysConf {
    clock: Clock {
      source: Main(12_000_000),
      pll: PLL0 {
        enabled: true,
        m: 50,
        n: 3,
        divisor: 4,
      }
    },
    enable_timer: true,
  }
};

#[no_split_stack]
#[no_mangle]
#[start]
pub extern fn main() {
  platform.configuration.setup();

  loop {
    unsafe { asm!("nop") }
  }
}
