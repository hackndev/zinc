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
use core::slice::ImmutableVector;
use core::mem::zeroed;

use super::font_small_7;
use super::LCD;
use drivers::chario::CharIO;
use hal::timer::Timer;
use hal::pin::GPIO;
use hal::spi::SPI;

pub struct C12332<'a, S, T, P> {
  spi: &'a S,
  timer: &'a T,

  dc:    &'a P,
  cs:    &'a P,
  reset: &'a P,

  videobuf: [cell::Cell<u8>, ..512],

  font: &'static [u8],
  char_x: cell::Cell<u32>,
  char_y: cell::Cell<u32>,
}

impl<'a, S: SPI, T: Timer, P: GPIO> C12332<'a, S, T, P> {
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

  pub fn set_pixel(&self, x: i32, y: i32, color: u16) {
    if x > 127 || y > 31 || x < 0 || y < 0 {
      return
    }

    let index = x + (y/8) * 128;
    if color == 0 {
      self.videobuf[index as uint].set(
        self.videobuf[index as uint].get() & !(1i32 << (y%8i32) as uint) as u8);
    } else {
      self.videobuf[index as uint].set(
        self.videobuf[index as uint].get() | (1 << ((y%8) as uint)));
    }
  }

  pub fn character(&self, x: i32, y: i32, c: u8) {
    let hor: u8;
    let vert: u8;
    let offset: u8;
    let bpl: u8;
    let mut j: u8;
    let mut i: u8;
    let mut b: u8;
    let zeichen: &[u8];
    let mut z: u8;
    let w: u8;
    let width: u32 = 128;
    let height: u32 = 32;

    if (c < 31) || (c > 127) {
      return;
    }

    // read font parameter from start of array
    offset = self.font[0];                    // bytes / char
    hor = self.font[1];                       // get hor size of font
    vert = self.font[2];                      // get vert size of font
    bpl = self.font[3];                       // bytes per line

    let mut char_x: u32 = self.char_x.get();
    let mut char_y: u32 = self.char_y.get();

    if char_x + hor as u32 > width {
      char_x = 0;
      char_y = char_y + vert as u32;
      if char_y >= height - self.font[2] as u32 {
        char_y = 0;
      }
    }

    let start: uint = ((c - 32) as uint * offset as uint) + 4;
    let end: uint = start + offset as uint;
    zeichen = self.font.slice(start, end);
    // zeichen = &self.font[]; // start of char bitmap
    w = zeichen[0];                          // width of actual char
    // construct the char into the buffer
    j = 0;
    while j < vert {
      i = 0;
      while i < hor {
        z =  zeichen[(bpl * i + ((j & 0xF8) >> 3)+1) as uint];
        b = 1 << ((j & 0x07) as uint);
        if ( z & b ) == 0x00 {
          self.set_pixel(x+i as i32, y+j as i32, 0);
        } else {
          self.set_pixel(x+i as i32, y+j as i32, 1);
        }
        i += 1;
      }
      j += 1;
    }

    char_x += w as u32;

    self.char_x.set(char_x);
    self.char_y.set(char_y);
  }
}

impl<'a, S: SPI, T: Timer, P: GPIO> LCD for C12332<'a, S, T, P> {
  fn flush(&self) {
    let mut i: uint = 0;

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
    let mut i = 0;
    while i < 512 {
      self.videobuf[i].set(0);
      i += 1;
    }
  }

  fn pixel(&self, x: i32, y: i32, color: u16) {
    self.set_pixel(x, y, color);
  }
}

impl<'a, S: SPI, T: Timer, P: GPIO> CharIO for C12332<'a, S, T, P> {
  fn putc(&self, value: char) {
    let height: u32 = 32;
    if value == '\n' {
      self.char_x.set(0);
      self.char_y.set(self.char_y.get() + self.font[2] as u32);
      if self.char_y.get() >= height - self.font[2] as u32 {
          self.char_y.set(0);
      }
    } else {
      self.character(self.char_x.get() as i32, self.char_y.get() as i32, value as u8);
    }
  }
}
