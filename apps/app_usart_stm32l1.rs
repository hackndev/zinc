#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

#[no_mangle]
pub unsafe fn main() {
  use core::option;
  use zinc::drivers::chario::CharIO;
  use zinc::hal;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32l1::{init, pin, usart};
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

  let uart = usart::Usart::new(usart::USART1, 115200, usart::WordLen8bits,
    hal::uart::Disabled, usart::StopBit1bit, &sys_clock);
  uart.puts("Hello, world\n");

  loop {
  }
}
