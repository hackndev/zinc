#![crate_id="app"]
#![crate_type="rlib"]
#![no_std]

extern crate zinc;

#[cfg(mcu_lpc17xx)] use zinc::boards::mbed_lpc1768;
#[cfg(mcu_stm32f4)] use zinc::boards::stm32f4discovery;
use zinc::hal::gpio::GPIOConf;
use zinc::hal::init::SysConf;
use zinc::hal::timer::{TimerConf, Timer};

#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;
#[cfg(mcu_stm32f4)] use zinc::hal::stm32f4;

struct Platform {
  configuration: SysConf,
  led1: GPIOConf,
  led2: GPIOConf,
  timer: TimerConf,
}

#[cfg(mcu_lpc17xx)]
static platform: Platform = Platform {
  configuration: mbed_lpc1768::configuration,
  led1: mbed_lpc1768::led1,
  led2: mbed_lpc1768::led2,
  timer: TimerConf {
    timer: lpc17xx::timer::Timer1,
    counter: 25,
    divisor: 4,
  },
};

#[cfg(mcu_stm32f4)]
static platform: Platform = Platform {
  configuration: stm32f4discovery::configuration,
  led1: stm32f4discovery::led1,
  led2: stm32f4discovery::led2,
  timer: TimerConf {
    timer: stm32f4::timer::Timer2,
    counter: 84,  // FIXME(farcaller): note on DCKCFGR
  },
};

#[no_split_stack]
pub fn main() {
  platform.configuration.setup();

  let led1 = platform.led1.setup();
  let led2 = platform.led2.setup();
  let timer = &platform.timer.setup() as &Timer;

  loop {
    led1.set_high();
    led2.set_low();
    timer.wait(1);
    led1.set_low();
    led2.set_high();
    timer.wait(1);
  }
}
