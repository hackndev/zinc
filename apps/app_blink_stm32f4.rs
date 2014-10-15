#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

#[no_mangle]
#[no_stack_check]
#[allow(unused_variable)]
#[allow(dead_code)]
pub unsafe fn main() {
  use zinc::hal::timer::Timer;
  use zinc::hal::stm32f4::{pin, timer};
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let led1 = pin::PinConf{
    port: pin::PortG,
    pin: 13u8,
    function: pin::GPIOOut
  };
  let led2 = pin::PinConf{
    port: pin::PortG,
    pin: 14u8,
    function: pin::GPIOOut
  };
  led1.setup();
  led2.setup();

  let timer = timer::Timer::new(timer::Timer2, 25u32);

  loop {
    led1.set_high();
    led2.set_low();
    timer.wait_ms(300);
    led1.set_low();
    led2.set_high();
    timer.wait_ms(300);
  }
}
