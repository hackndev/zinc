#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

//! Sample application for
//! BlueNRG communication over SPI in
//! X-NUCLEO-IDB04A1 on NUCLEO-L152RE

extern crate core;
extern crate zinc;

#[no_mangle]
pub unsafe fn main() {
  use zinc::drivers::chario::CharIO;
  use zinc::hal;
  //use zinc::hal::pin::Gpio;
  use zinc::hal::spi::Spi;
  use zinc::hal::stm32l1::{init, pin, spi, usart};

  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let sys_clock = init::ClockConfig::new_default();
  sys_clock.setup();

  let _usart_tx = pin::Pin::new(pin::PortA, 2,
    pin::AltFunction(
      pin::AfUsart1_Usart2_Usart3,
      pin::OutPushPull,
      pin::VeryLow),
    pin::PullNone);

  let uart = usart::Usart::new(usart::Usart2, 38400, usart::WordLen8bits,
    hal::uart::Disabled, usart::StopBit1bit, &sys_clock);
  uart.puts("BlueNRG test app for STM32L1\n");

  let _spi_clock = pin::Pin::new(pin::PortA, 5,
    pin::AltFunction(pin::AfSpi1_Spi2, pin::OutPushPull, pin::VeryLow),
    pin::PullNone);

  let _spi_in = pin::Pin::new(pin::PortA, 6,
    pin::AltFunction(pin::AfSpi1_Spi2, pin::OutPushPull, pin::VeryLow),
    pin::PullNone);

  let _spi_out = pin::Pin::new(pin::PortA, 7,
    pin::AltFunction(pin::AfSpi1_Spi2, pin::OutPushPull, pin::VeryLow),
    pin::PullNone);

  let _spi = spi::Spi::new(spi::Spi1, spi::SpiFullDuplex, spi::SpiMaster,
    spi::SpiData8b, spi::SpiMsbFirst, 1, spi::SpiEdge1, spi::SpiLowPolarity);

  uart.puts("SPI connection established\n");

  loop {}
}
