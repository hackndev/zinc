#![feature(no_std, core, start)]
#![no_std]

extern crate core;
extern crate zinc;

#[start]
fn start(_: isize, _: *const *const u8) -> isize {
    main();
    0
}

pub unsafe fn main() {
  use core::option::Option;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32l1::{init, pin, timer};
  use zinc::hal::timer::Timer;
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let sys_clock = init::ClockConfig {
    source: init::SystemClockSource::SystemClockHSI,
    ahb_shift: 0,
    apb1_shift: 0,
    apb2_shift: 0,
    mco: Option::None,
  };
  sys_clock.setup();

  let led1 = pin::Pin::new(pin::Port::PortA, 5,
    pin::Mode::GpioOut(pin::OutputType::OutPushPull, pin::Speed::VeryLow),
    pin::PullType::PullNone);

  // TODO(kvark): why doesn't "sys_clock.get_apb1_frequency()" work better?
  let timer_clock = sys_clock.source.frequency();
  let timer = timer::Timer::new(timer::TimerPeripheral::Timer2, timer_clock/1000, 0);

  loop {
    led1.set_high();
    timer.wait_ms(1);
    led1.set_low();
    timer.wait_ms(1);
  }
}
