#![feature(core_intrinsics)]
#![feature(plugin, no_std)]
#![no_std]
#![plugin(macro_zinc)]


extern crate zinc;

use zinc::hal::timer::Timer;
use zinc::hal::pin::Gpio;
use zinc::hal::stm32f4::{pin, timer, spi};
use zinc::hal::spi::Spi;
use core::intrinsics::abort;

#[zinc_main]
pub fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

  let led1 = pin::Pin::new (
      pin::Port::PortD,
      13u8,
      pin::Mode::GpioOut(pin::OutputType::OutPushPull, pin::Speed::VeryLow),
      pin::PullType::PullNone
  );

  let spi1Sck = pin::Pin::new (
      pin::Port::PortA,
      5u8,
      pin::Mode::AltFunction(pin::AltMode::AfSpi1_Spi2, pin::OutputType::OutPushPull, pin::Speed::High),
      pin::PullType::PullNone
  );

  let spi1Miso = pin::Pin::new (
      pin::Port::PortA,
      6u8,
      pin::Mode::AltFunction(pin::AltMode::AfSpi1_Spi2, pin::OutputType::OutPushPull, pin::Speed::High),
      pin::PullType::PullNone
  );

  let spi1Mosi = pin::Pin::new (
      pin::Port::PortA,
      7u8,
      pin::Mode::AltFunction(pin::AltMode::AfSpi1_Spi2, pin::OutputType::OutPushPull, pin::Speed::High),
      pin::PullType::PullNone
  );

  let spi1Ss = pin::Pin::new(
      pin::Port::PortA,
      4u8,
      pin::Mode::GpioOut(pin::OutputType::OutPushPull, pin::Speed::VeryLow),
      pin::PullType::PullNone
  );

  let timer = timer::Timer::new(timer::TimerPeripheral::Timer2, 25u32);

  let spi1 = match spi::Spi::new(
      spi::Peripheral::Spi1,
      spi::Direction::FullDuplex,
      spi::Role::Master,
      spi::DataSize::U8,
      spi::DataFormat::MsbFirst,
      6) {
          Ok(e) => e,
          _ => unsafe {abort ()},
      };

  spi1Ss.set_high();

  loop {
    led1.set_high();
    spi1Ss.set_low();
    spi1.write('H' as u8);
    spi1.write('e' as u8);
    spi1.write('l' as u8);
    spi1.write('l' as u8);
    spi1.write('o' as u8);
    spi1.write('!' as u8);
    //led1.set_low();
    //spi1Ss.set_high();
  }
}
