#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

//! Sample application for BlueNRG communication over SPI in X-NUCLEO-IDB04A1
//! extension board for NUCLEO-L152RE

#[phase(plugin, link)]
extern crate core;
extern crate zinc;

use core::intrinsics::abort;
// "curious module hack" - a workaround for `fmt` module being looked in `std`
// by the `write!` macro
mod std {
  pub use core::fmt;
}

//TODO(kvark): temporary `u8 -> str` conversion until #235 is resolved
fn map_byte(s: u8) -> (&'static str, &'static str) {
  fn map_hex(h: u8) -> &'static str {
      match h {
        0x0 => "0",
        0x1 => "1",
        0x2 => "2",
        0x3 => "3",
        0x4 => "4",
        0x5 => "5",
        0x6 => "6",
        0x7 => "7",
        0x8 => "8",
        0x9 => "9",
        0xA => "A",
        0xB => "B",
        0xC => "C",
        0xD => "D",
        0xE => "E",
        0xF => "F",
        _ => "",
      }
  }
  (map_hex(s>>4), map_hex(s&0xF))
}

#[no_mangle]
pub unsafe fn main() {
  use core::fmt::FormatWriter;
  use core::result::Result;
  use zinc::drivers::bluenrg;
  use zinc::hal;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32l1::{init, pin, spi, usart};

  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let sys_clock = init::ClockConfig::new_default();
  sys_clock.setup();

  let _usart_tx = pin::Pin::new(pin::Port::PortA, 2,
    pin::Mode::AltFunction(
      pin::AltMode::AfUsart1_Usart2_Usart3,
      pin::OutputType::OutPushPull,
      pin::Speed::VeryLow),
    pin::PullType::PullNone);

  let mut uart = usart::Usart::new(usart::UsartPeripheral::Usart2, 38400, usart::WordLen::WordLen8bits,
    hal::uart::Parity::Disabled, usart::StopBit::StopBit1bit, &sys_clock);
  let _ = write!(&mut uart, "BlueNRG test app for STM32L1\n");

  let _spi_clock = pin::Pin::new(pin::Port::PortB, 3,
    pin::Mode::AltFunction(pin::AltMode::AfSpi1_Spi2, pin::OutputType::OutPushPull, pin::Speed::Medium),
    pin::PullType::PullDown);

  let _spi_in = pin::Pin::new(pin::Port::PortA, 6,
    pin::Mode::AltFunction(pin::AltMode::AfSpi1_Spi2, pin::OutputType::OutPushPull, pin::Speed::Medium),
    pin::PullType::PullNone);

  let _spi_out = pin::Pin::new(pin::Port::PortA, 7,
    pin::Mode::AltFunction(pin::AltMode::AfSpi1_Spi2, pin::OutputType::OutPushPull, pin::Speed::Medium),
    pin::PullType::PullNone);

  let spi_csn = pin::Pin::new(pin::Port::PortA, 1,
    pin::Mode::GpioOut(pin::OutputType::OutPushPull, pin::Speed::Medium),
    pin::PullType::PullUp);
  spi_csn.set_high();

  let spi = spi::Spi::new(spi::Peripheral::Spi1, spi::Direction::FullDuplex,
    spi::Role::Master, spi::DataSize::U8, spi::DataFormat::MsbFirst, 1).
    unwrap_or_else(|_| {
      let _ = write!(&mut uart, "SPI failed to initialize");
      abort()
    });

  let bnrg_reset = pin::Pin::new(pin::Port::PortA, 8,
    pin::Mode::GpioOut(pin::OutputType::OutPushPull, pin::Speed::VeryLow),
    pin::PullType::PullUp);

  bnrg_reset.set_low();
  let _ = write!(&mut uart, "SPI created, status = {}\n",
    map_byte(spi.get_status()));
  bnrg_reset.set_high();

  let blue = bluenrg::BlueNrg::new(spi_csn, spi);

  let _ = match blue.wakeup(100) {
    Result::Ok((size_write, size_read)) => write!(&mut uart,
      "BlueNRG is ready, write size = {}, read size = {}\n",
      map_byte(size_write as u8), map_byte(size_read as u8)),
    Result::Err(bluenrg::Error::Sleeping) => write!(&mut uart,
      "BlueNRG is sleeping\n"),
    Result::Err(bluenrg::Error::Allocating) => write!(&mut uart,
      "BlueNRG is allocating buffers\n"),
    Result::Err(bluenrg::Error::Unknown(status)) => write!(&mut uart,
      "BlueNRG unknown status = {}\n", map_byte(status)),
    Result::Err(bluenrg::Error::BufferSize(_)) => write!(&mut uart, ""),
  };

  loop {}
}
