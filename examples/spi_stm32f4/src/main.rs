#![feature(plugin, no_std)]
#![no_std]
#![plugin(macro_zinc)]

extern crate zinc;

use zinc::hal::timer::Timer;
use zinc::hal::pin::Gpio;
use zinc::hal::stm32f4::{pin, timer};

#[zinc_main]
pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let led1 = pin::Pin {
    port: pin::Port::PortD,
    pin: 13u8,
    function: pin::Function::GPIOOut
  };
  let led2 = pin::Pin {
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
