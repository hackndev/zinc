#![feature(plugin, start)]
#![no_std]
#![plugin(macro_zinc)]

extern crate zinc;

use zinc::hal::timer::Timer;
use zinc::hal::pin::Gpio;
use zinc::hal::stm32f7::{pin, timer};

#[zinc_main]
pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  // Turn off the LCD backlight (PK3)
  let backlight = pin::Pin {
    port: pin::Port::PortK,
    pin: 3,
    function: pin::Function::GPIOOut
  };
  backlight.setup();
  backlight.set_low();

  // The STM32F7 Discovery board LED is on PI1
  let led = pin::Pin {
    port: pin::Port::PortI,
    pin: 1,
    function: pin::Function::GPIOOut
  };
  led.setup();

  let timer = timer::Timer::new(timer::TimerPeripheral::Timer2, 16);

  loop {
    led.set_high();
    timer.wait_ms(100);
    led.set_low();
    timer.wait_ms(100);
    led.set_high();
    timer.wait_ms(100);
    led.set_low();
    timer.wait_ms(700);
  }
}
