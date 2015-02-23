#![feature(plugin, no_std, core)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;

#[no_mangle]
pub unsafe fn main() {
  use zinc::drivers::chario::CharIO;
  use zinc::hal;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32l1::{init, pin, usart};

  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let sys_clock = init::ClockConfig::new_default();
  sys_clock.setup();

  let _pin_tx = pin::Pin::new(pin::Port::PortA, 2,
    pin::Mode::AltFunction(
      pin::AltMode::AfUsart1_Usart2_Usart3,
      pin::OutputType::OutPushPull,
      pin::Speed::VeryLow),
    pin::PullType::PullNone);

  let led1 = pin::Pin::new(pin::Port::PortA, 5,
    pin::Mode::GpioOut(pin::OutputType::OutPushPull, pin::Speed::VeryLow),
    pin::PullType::PullNone);

  led1.set_low();

  let uart = usart::Usart::new(usart::UsartPeripheral::Usart2, 38400, usart::WordLen::WordLen8bits,
    hal::uart::Parity::Disabled, usart::StopBit::StopBit1bit, &sys_clock);
  uart.puts("Hello, world\n");

  led1.set_high();

  loop {}
}
