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
  use zinc::hal::stm32l1::{init, pin, timer, usart};
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

  let led1 = pin::Pin::new(pin::PortA, 5,
    pin::GpioOut(pin::OutPushPull, pin::VeryLow),
    pin::PullNone);

  // TODO(kvark): why doesn't "sys_clock.get_apb1_frequency()" work better?
  let timer_clock = sys_clock.source.frequency();
  let timer = timer::Timer::new(timer::Timer2, timer_clock/1000, 0);

  let uart = usart::Usart::new(usart::USART1, 115200, usart::WordLen8bits,
    hal::uart::Disabled, usart::StopBit1bit, &sys_clock);
  uart.puts("Hello, world\n");

  loop {
    led1.set_high();
    timer.wait_ms(1);
    led1.set_low();
    timer.wait_ms(1);
  }
}
