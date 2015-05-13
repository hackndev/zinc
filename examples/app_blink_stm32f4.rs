#![feature(no_std, core, start)]
#![no_std]

extern crate core;
extern crate zinc;

use zinc::hal::timer::Timer;
use zinc::hal::stm32f4::{pin, timer};

#[start]
fn start(_: isize, _: *const *const u8) -> isize {
    main();
    0
}

pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let led1 = pin::PinConf{
    port: pin::Port::PortD,
    pin: 13u8,
    function: pin::Function::GPIOOut
  };
  let led2 = pin::PinConf{
    port: pin::Port::PortD,
    pin: 14u8,
    function: pin::Function::GPIOOut
  };
  led1.setup();
  led2.setup();

  let timer = timer::Timer::new(timer::TimerPeripheral::Timer2, 25u32);

  loop {
    led1.set_high();
    led2.set_low();
    timer.wait_ms(300);
    led1.set_low();
    led2.set_high();
    timer.wait_ms(300);
  }
}
