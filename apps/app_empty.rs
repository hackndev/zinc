#![feature(asm)]
#![crate_id="app"]
#![crate_type="rlib"]
#![no_std]

extern crate zinc;

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx::init::{SysConf, Clock, Main, PLL0};
#[cfg(mcu_stm32f4)] use zinc::hal::stm32f4::init::{SysConf, ClockConf};
#[cfg(mcu_stm32f4)] use zinc::hal::stm32f4::init::{SystemClockPLL,PLLConf, PLLClockHSE};

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
  }
};

#[cfg(mcu_stm32f4)]
static platform: Platform = Platform {
  configuration: SysConf {
    clock: ClockConf {
      source: SystemClockPLL(PLLConf {
        source: PLLClockHSE(8_000_000),
        m: 8,
        n: 316,
        p: 2,
        q: 7,
      })
    }
  }
};

#[no_split_stack]
pub fn main() {
  platform.configuration.setup();

  loop {
    unsafe { asm!("nop") }
  }
}
