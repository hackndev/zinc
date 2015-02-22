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

/*!
Driver for C12332 LCD.

C12332 is black&white LCD, the only supported color value is `1`. The LCD is
buffered in driver memory.

The driver uses SPI bus for output only, it never reads back from SPI, which
might be an issue for any other peripheral sharing the same SPI bus.
*/

use core::cell;
use core::slice::SliceExt;
use core::mem::zeroed;
use core::iter::range;

use super::font_small_7;
use super::LCD;
use drivers::chario::CharIO;
use hal::timer::Timer;
use hal::pin::Gpio;
use hal::spi::Spi;

/// C12332 driver.
pub struct C12332<'a, S:'a, T:'a, P:'a> {
  spi: &'a S,
  timer: &'a T,

  dc:    &'a P,
  cs:    &'a P,
  reset: &'a P,

  videobuf: [cell::Cell<u8>; 512],

  font: &'static [u8],
  char_x: cell::Cell<u32>,
  char_y: cell::Cell<u32>,
}

impl<'a, S: Spi, T: Timer, P: Gpio> C12332<'a, S, T, P> {
  /// Creates a new C12332 driver instance.
  pub fn new(spi: &'a S, timer: &'a T, dc: &'a P, cs: &'a P,
      reset: &'a P) -> C12332<'a, S, T, P> {
    let lcd = C12332 {
      spi:   spi,
      timer: timer,
      dc:    dc,
      cs:    cs,
      reset: reset,

      videobuf: unsafe { zeroed() },

      font: font_small_7::FONT,
      char_x: cell::Cell::new(0),
      char_y: cell::Cell::new(0),
    };

    lcd.configure();

    lcd
  }

  fn configure(&self) {
    self.dc.set_low();
    self.cs.set_high();
    self.reset.set_low();
    self.timer.wait_us(50);
    self.reset.set_high();
    self.timer.wait_ms(5);

    self.wr_cmd(0xAE);   //  display off
    self.wr_cmd(0xA2);   //  bias voltage

    self.wr_cmd(0xA0);
    self.wr_cmd(0xC8);   //  colum normal

    self.wr_cmd(0x22);   //  voltage resistor ratio
    self.wr_cmd(0x2F);   //  power on
    //self.wr_cmd(0xA4);   //  LCD display ram
    self.wr_cmd(0x40);   // start line = 0
    self.wr_cmd(0xAF);     // display ON

    self.wr_cmd(0x81);   //  set contrast
    self.wr_cmd(0x17);   //  set contrast

    self.wr_cmd(0xA6);     // display normal

    (self as &LCD).flush();
  }

  fn wr_cmd(&self, cmd: u8) {
    self.dc.set_low();
    self.cs.set_low();
    self.spi.write(cmd);
    self.cs.set_high();
  }

  fn wr_dat(&self, cmd: u8) {
    self.dc.set_high();
    self.cs.set_low();
    self.spi.write(cmd);
    self.cs.set_high();
  }

  /// Sets an individual pixel.
  pub fn set_pixel(&self, x: u32, y: u32, color: u16) {
    if x > 127 || y > 31 {
      return
    }

    let index = (x + (y/8) * 128) as usize;
    if color == 0 {
      self.videobuf[index].set(
        self.videobuf[index].get() & !(1u8 << (y%8u32) as usize) as u8);
    } else {
      self.videobuf[index].set(
        self.videobuf[index].get() | (1 << ((y%8) as usize)));
    }
  }

  /// Prints a character to the display.
  pub fn character(&self, x: u32, y: u32, c: u8) {
    let width: u32 = 128;
    let height: u32 = 32;

    if (c < 31) || (c > 127) {
      return;
    }

    // read font parameter from start of array
    let offset = self.font[0];                    // bytes / char
    let hor = self.font[1];                       // get hor size of font
    let vert = self.font[2];                      // get vert size of font
    let bpl = self.font[3];                       // bytes per line

    let mut char_x: u32 = self.char_x.get();
    let mut char_y: u32 = self.char_y.get();

    if char_x + hor as u32 > width {
      char_x = 0;
      char_y = char_y + vert as u32;
      if char_y >= height - self.font[2] as u32 {
        char_y = 0;
      }
    }

    let start: usize = ((c - 32) as usize * offset as usize) + 4;
    let end: usize = start + offset as usize;
    let zeichen = &self.font[start..end];
    // zeichen = &self.font[]; // start of char bitmap
    let w = zeichen[0];                          // width of actual char
    // construct the char into the buffer
    for j in range(0, vert) {
      for i in range(0, hor) {
        let z: u8 =  zeichen[(bpl * i + ((j & 0xF8) >> 3)+1) as usize];
        let b: u8 = 1 << ((j & 0x07) as usize);
        if ( z & b ) == 0x00 {
          self.set_pixel(x+i as u32, y+j as u32, 0);
        } else {
          self.set_pixel(x+i as u32, y+j as u32, 1);
        }
      }
    }

    char_x += w as u32;

    self.char_x.set(char_x);
    self.char_y.set(char_y);
  }
}

impl<'a, S: Spi, T: Timer, P: Gpio> LCD for C12332<'a, S, T, P> {
  fn flush(&self) {
    let mut i: usize = 0;

    //page 0
    self.wr_cmd(0x00);      // set column low nibble 0
    self.wr_cmd(0x10);      // set column hi  nibble 0
    self.wr_cmd(0xB0);      // set page address  0
    self.dc.set_high();
    while i < 128 {
      self.wr_dat(self.videobuf[i].get());
      i += 1;
    }

    // page 1
    self.wr_cmd(0x00);      // set column low nibble 0
    self.wr_cmd(0x10);      // set column hi  nibble 0
    self.wr_cmd(0xB1);      // set page address  1
    self.dc.set_high();
    while i < 256 {
      self.wr_dat(self.videobuf[i].get());
      i += 1;
    }

    //page 2
    self.wr_cmd(0x00);      // set column low nibble 0
    self.wr_cmd(0x10);      // set column hi  nibble 0
    self.wr_cmd(0xB2);      // set page address  2
    self.dc.set_high();
    while i < 384 {
      self.wr_dat(self.videobuf[i].get());
      i += 1;
    }

    //page 3
    self.wr_cmd(0x00);      // set column low nibble 0
    self.wr_cmd(0x10);      // set column hi  nibble 0
    self.wr_cmd(0xB3);      // set page address  3
    self.dc.set_high();
    while i < 512 {
      self.wr_dat(self.videobuf[i].get());
      i += 1;
    }
  }

  fn clear(&self) {
    for i in range(0usize, 512) {
      self.videobuf[i].set(0);
    }
  }

  fn pixel(&self, x: u32, y: u32, color: u16) {
    self.set_pixel(x, y, color);
  }
}

impl<'a, S: Spi, T: Timer, P: Gpio> CharIO for C12332<'a, S, T, P> {
  fn putc(&self, value: char) {
    let height: u32 = 32;
    if value == '\n' {
      self.char_x.set(0);
      self.char_y.set(self.char_y.get() + self.font[2] as u32);
      if self.char_y.get() >= height - self.font[2] as u32 {
          self.char_y.set(0);
      }
    } else {
      self.character(self.char_x.get(), self.char_y.get(), value as u8);
    }
  }
}
