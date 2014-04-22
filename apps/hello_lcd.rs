#![crate_type="staticlib"]
#![no_std]

extern crate zinc;

use zinc::hal;
use zinc::drivers;
use zinc::interfaces;

mod img;

#[no_split_stack]
#[no_mangle]
#[start]
pub extern fn main() {
  hal::platform_init();

  let led1 = hal::gpio::OutGPIO::new(1, 18);
  let led2 = hal::gpio::OutGPIO::new(1, 20);

  let uart_dev = hal::lpc1768::uart::UART::new(hal::lpc1768::uart::UART0,
                 9600, hal::lpc1768::uart::WordLen8bits,
                 hal::lpc1768::uart::ParityNone, hal::lpc1768::uart::StopBit1bit);

  hal::lpc1768::debug::set_uart_debugger(&uart_dev as *hal::lpc1768::uart::UART);

  let uart = &uart_dev as &interfaces::chario::CharIO;

  uart.puts("hello uart");

  led1.set_high();

  // let spi = hal::lpc1768::ssp::SSP::new(hal::lpc1768::ssp::SSP1, 20_000000, 8, 3, false);
  // let mut c12332_lcd = drivers::lcd::c12332::C12332::new(&spi);
  // let lcd = &mut c12332_lcd as &mut interfaces::lcd::LCD;

  let spi = hal::lpc1768::ssp::SSP::new(hal::lpc1768::ssp::SSP1, 6_000000, 8, 0, false);
  let mut ili_lcd = drivers::lcd::ili9341::ILI9341::new(&spi);
  let lcd = &mut ili_lcd as &mut interfaces::lcd::LCD;

  lcd.fillrect(10, 10, 40, 40, interfaces::lcd::ColorBlue as u16);
  lcd.draw_image(img::image_width, img::image_height, img::image_data);
  // lcd.clear();
  // lcd.puts("hello lcd");
  // lcd.flush();

  hal::lpc1768::debug::d().puts("hello debug");

  loop {
    led1.set_high();
    led2.set_low();
    hal::timer::wait(1);
    led1.set_low();
    led2.set_high();
    hal::timer::wait(1);
  }
}
