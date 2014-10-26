#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

#[no_mangle]
pub unsafe fn main() {
  use core::option;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32l1::{init, pin, timer};
  use zinc::hal::timer::Timer;
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let sys_clock = init::ClockConfig {
    source: init::SystemClockHSI,
    ahb_shift: 0,
    apb1_shift: 0,
    apb2_shift: 0,
    mco: option::None,
  };
  sys_clock.setup();

  let mcu_clock = sys_clock.source.frequency();

  let led1 = pin::Pin::new(pin::PortA, 5,
    pin::GpioOut(pin::OutPushPull, pin::VeryLow),
    pin::PullNone);

  let timer = timer::Timer::new(timer::Timer2, mcu_clock/1000, 0);

  loop {
    led1.set_high();
    timer.wait_ms(1);
    led1.set_low();
    timer.wait_ms(1);
  }
}
