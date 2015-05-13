// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Driver for the ILI9341 LCD.

use super::LCD;
use drivers::chario::CharIO;
use hal::timer::Timer;
use hal::pin::Gpio;
use hal::spi::Spi;

/// ILI9341 driver.
pub struct ILI9341<'a, S:'a, T:'a, P:'a> {
  spi: &'a S,
  timer: &'a T,
  dc: &'a P,
  cs: &'a P,
  reset: &'a P,
  // backlight: gpio::OutGPIO,
}

impl<'a, S: Spi, T: Timer, P: Gpio> ILI9341<'a, S, T, P> {
  /// Creates a new ILI9341 driver instance.
  pub fn new(spi: &'a S, timer: &'a T, dc: &'a P, cs: &'a P, reset: &'a P)
    -> ILI9341<'a, S, T, P> {
    let lcd = ILI9341 {
      spi: spi,
      timer: timer,
      dc: dc,
      cs: cs,
      reset:reset,
      // dc: gpio::OutGPIO::new(0, 24),
      // cs: gpio::OutGPIO::new(0, 16),
      // reset: gpio::OutGPIO::new(0, 23),
      // backlight: gpio::OutGPIO::new(0, 999),
    };

    // FIXME(farcaller): SPI uses MSB, SCL rising
    lcd.configure();

    lcd
  }

  fn configure(&self) {
    self.cs.set_high();
    self.dc.set_high();

    self.reset.set_low();
    self.timer.wait_ms(10);
    self.reset.set_high();

    self.verify_id(); // this fails :)
    self.verify_id(); // TODO(farcaller): verify that this didn't fail or bail out
    self.verify_id(); // and this as well

    self.set_power_control_a();
    self.set_power_control_b();
    self.driver_timing_control_a();
    self.driver_timing_control_b();
    self.power_on_sequence_control();
    self.pump_ratio_control();
    self.power_control_1();
    self.power_control_2();
    self.vcom_control_1();
    self.everything_else();
  }

  fn verify_id(&self) -> bool {
    let mut data: [u8; 3] = [0, 0, 0];
    let id: [u8; 3] = [0x00, 0x93, 0x41];

    for i in 0 .. 3 {
      data[i] = self.read_register(0xd3, (i+1) as u8);
      if data[i] != id[i] {
        return false;
      }
    }

    true
  }

  fn set_power_control_a(&self) {
    self.send_cmd(0xcb);
    self.write_data(0x39);
    self.write_data(0x2c);
    self.write_data(0x00);
    self.write_data(0x34); // REG_VD = 0b100 = Vcore 1.6V
    self.write_data(0x02); // VBC    = 0b010 = DDVDH 5.6V
  }

  fn set_power_control_b(&self) {
    self.send_cmd(0xcf);
    self.write_data(0x00);
    self.write_data(0xc1); // TODO(farcaller): according to the spec this is 0x81
    self.write_data(0x30); // ESD protection enabled
  }

  fn driver_timing_control_a(&self) {
    self.send_cmd(0xe8);
    self.write_data(0x85); // Non-overlap timing control = 1 unit
    self.write_data(0x00); // EQ timing = 1 unit; CR timing = 1 unit
    self.write_data(0x78); // Pre-chanrge timing = 2 unit
  }

  fn driver_timing_control_b(&self) {
    self.send_cmd(0xea);
    self.write_data(0x00); // 0 units EQ to GND, DDVDH
    self.write_data(0x00);
  }

  fn power_on_sequence_control(&self) {
    self.send_cmd(0xed);
    self.write_data(0x64); // CP1,CP2,CP3 soft start keep 1 frame
    self.write_data(0x03); // Vcl 1st frame enable; DDVDH 4th frame enable
    self.write_data(0x12); // Vgh 2nd frame enable; Vgl   3rd frame enable
    self.write_data(0x81); // DDVDH enhance mode enabled
  }

  fn pump_ratio_control(&self) {
    self.send_cmd(0xf7);
    self.write_data(0x20); // DDVDH = 2xVCI
  }

  fn power_control_1(&self) {
    self.send_cmd(0xc0);
    self.write_data(0x23); // GVDD = 4.6V
  }

  fn power_control_2(&self) {
    self.send_cmd(0xc1);
    self.write_data(0x10); // another wtf. I guess it's DDVDH =  VCI*2
                           //                           VGH   =  VCI*7
                           //                           VGL   = -VCI*4
  }

  fn vcom_control_1(&self) {
    self.send_cmd(0xc5);
    self.write_data(0x3e); // VCOMH = 4.25V
    self.write_data(0x28); // VCOML = -1.5V
  }

  fn everything_else(&self) {
    self.send_cmd(0xC7);
    self.write_data(0x86);

    self.send_cmd(0x36);
    self.write_data(0x48);

    self.send_cmd(0x3A);
    self.write_data(0x55);

    self.send_cmd(0xB1);
    self.write_data(0x00);
    self.write_data(0x18);

    self.send_cmd(0xB6);
    self.write_data(0x08);
    self.write_data(0x82);
    self.write_data(0x27);

    self.send_cmd(0xF2);
    self.write_data(0x00);

    self.send_cmd(0x26);
    self.write_data(0x01);

    self.send_cmd(0xE0);
    self.write_data(0x0F);
    self.write_data(0x31);
    self.write_data(0x2B);
    self.write_data(0x0C);
    self.write_data(0x0E);
    self.write_data(0x08);
    self.write_data(0x4E);
    self.write_data(0xF1);
    self.write_data(0x37);
    self.write_data(0x07);
    self.write_data(0x10);
    self.write_data(0x03);
    self.write_data(0x0E);
    self.write_data(0x09);
    self.write_data(0x00);

    self.send_cmd(0xE1);
    self.write_data(0x00);
    self.write_data(0x0E);
    self.write_data(0x14);
    self.write_data(0x03);
    self.write_data(0x11);
    self.write_data(0x07);
    self.write_data(0x31);
    self.write_data(0xC1);
    self.write_data(0x48);
    self.write_data(0x08);
    self.write_data(0x0F);
    self.write_data(0x0C);
    self.write_data(0x31);
    self.write_data(0x36);
    self.write_data(0x0F);

    self.send_cmd(0x11);
    self.timer.wait_ms(120);

    self.send_cmd(0x29);
    self.send_cmd(0x2c);
  }

  fn read_register(&self, addr: u8, param: u8) -> u8 {
    self.send_cmd(0xd9);
    self.write_data(0x10 + param);

    self.dc.set_low();
    self.cs.set_low();
    self.spi.transfer(addr);
    self.dc.set_high();
    let data = self.spi.transfer(0);
    self.cs.set_high();

    data
  }

  #[inline(never)]
  fn send_cmd(&self, index: u8) {
    self.dc.set_low();
    self.cs.set_low();
    self.spi.transfer(index);
    self.cs.set_high();
  }

  #[inline(never)]
  fn write_data(&self, data: u8) {
    self.dc.set_high();
    self.cs.set_low();
    self.spi.transfer(data);
    self.cs.set_high();
  }

  fn send_data(&self, data: u16) {
    let data1: u8 = (data >> 8) as u8;
    let data2: u8 = (data & 0xff) as u8;
    self.dc.set_high();
    self.cs.set_low();
    self.spi.transfer(data1);
    self.spi.transfer(data2);
    self.cs.set_high();
  }

  fn set_col(&self, start: u16, end: u16) {
      self.send_cmd(0x2a);
      self.send_data(start);
      self.send_data(end);
  }

  fn set_page(&self, start: u16, end: u16) {
      self.send_cmd(0x2b);
      self.send_data(start);
      self.send_data(end);
  }

  fn do_clear(&self) {
    self.set_col(0, 239);
    self.set_page(0, 319);
    self.send_cmd(0x2c);

    self.dc.set_high();
    self.cs.set_low();
    for _ in 0..38400 {
      self.spi.transfer(0);
      self.spi.transfer(0);
      self.spi.transfer(0);
      self.spi.transfer(0);
    }
    self.cs.set_high();
  }

  fn do_pixel(&self, x: u32, y: u32, color: u16) {
    self.set_col(x as u16, x as u16);
    self.set_page(y as u16, y as u16);
    self.send_cmd(0x2c);
    self.send_data(color);
  }
}

impl<'a, S: Spi, T: Timer, P: Gpio> LCD for ILI9341<'a, S, T, P> {
  fn clear(&self) {
    self.do_clear();
  }
  fn flush(&self) {}
  fn pixel(&self, x: u32, y: u32, color: u16) {
    self.do_pixel(x, y, color);
  }
}

impl<'a, S: Spi, T: Timer, P: Gpio> CharIO for ILI9341<'a, S, T, P> {
  fn putc(&self, _: char) {
    // TODO(farcaller): implement
  }
}
