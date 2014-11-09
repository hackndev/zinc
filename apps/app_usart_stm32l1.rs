#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

#[no_mangle]
pub unsafe fn main() {
  use core::option;
  use core::default;
  use zinc::drivers::chario::CharIO;
  use zinc::hal;
  use zinc::hal::stm32l1::{init, usart};
  use zinc::hal::pin::Gpio; //temp
  use zinc::hal::stm32l1::pin; //temp
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let sys_clock = init::ClockConfig {
    source: default::Default::default(),
    ahb_shift: 0,
    apb1_shift: 0,
    apb2_shift: 0,
    mco: option::None,
  };
  sys_clock.setup();

  let _pin_tx = pin::Pin::new(pin::PortA, 2,
    pin::AltFunction(
      pin::AfUsart1_Usart2_Usart3,
      pin::OutPushPull,
      pin::VeryLow),
    pin::PullNone);

  let led1 = pin::Pin::new(pin::PortA, 5,
    pin::GpioOut(pin::OutPushPull, pin::VeryLow),
    pin::PullNone);

  led1.set_low();

  let uart = usart::Usart::new(usart::USART2, 38400, usart::WordLen8bits,
    hal::uart::Disabled, usart::StopBit1bit, &sys_clock);
  uart.puts("Hello, world\n");

  led1.set_high();

  loop {
  }
}
