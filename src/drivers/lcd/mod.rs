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

//! Drivers for TFT LCDs.

use core::option::{Some, None};
use core::iter::{Iterator, range};

use drivers::chario::CharIO;

#[cfg(cfg_mcu_has_spi)] pub mod c12332;
#[cfg(cfg_mcu_has_spi, FIXME_BROKEN)] pub mod ili9341;
pub mod font_small_7;

/// LCD provides a generic interface to a TFT LCD peripheral.
///
/// It provides generic methods for drawing primitives and bitmaps based on
/// `pixel` to set a pixel.
///
/// LCD does not flush buffers automatically, user must call `flush` after the
/// drwaing sequence to actually display the data on screen.
pub trait LCD : CharIO {
  /// Clears the screen.
  fn clear(&self);

  /// Flushes the internal buffer to screen, where applicable.
  fn flush(&self);

  /// Sets one pixel color. The actual color bits are driver-specific.
  fn pixel(&self, x: u32, y: u32, color: u16);

  /// Draws a line from xy0 to xy1.
  fn line(&self, x0_b: u32, y0_b: u32, x1: u32, y1: u32, color: u16) {
    let mut x0: u32 = x0_b;
    let mut y0: u32 = y0_b;
    let mut dx: u32;
    let mut dy: u32;
    let mut dx_sym: u32;
    let mut dy_sym: u32;
    let mut dx_x2: u32;
    let mut dy_x2: u32;
    let mut di: i32;

    dx = x1-x0;
    dy = y1-y0;

    if dx > 0 {
      dx_sym = 1;
    } else {
      dx_sym = -1;
    }

    if dy > 0 {
      dy_sym = 1;
    } else {
      dy_sym = -1;
    }

    dx = dx_sym*dx;
    dy = dy_sym*dy;

    dx_x2 = dx*2;
    dy_x2 = dy*2;

    if dx >= dy {
      di = (dy_x2 - dx) as i32;
      while x0 != x1 {
        self.pixel(x0, y0, color);
        x0 += dx_sym;
        if di < 0 {
          di += dy_x2 as i32;
        } else {
          di += (dy_x2 - dx_x2) as i32;
          y0 += dy_sym;
        }
      }
      self.pixel(x0, y0, color);
    } else {
      di = (dx_x2 - dy) as i32;
      while y0 != y1 {
        self.pixel(x0, y0, color);
        y0 += dy_sym;
        if di < 0 {
          di += dx_x2 as i32;
        } else {
          di += (dx_x2 - dy_x2) as i32;
          x0 += dx_sym;
        }
      }
      self.pixel(x0, y0, color);
    }
  }

  /// Draws a rectangle.
  fn rect(&self, x0: u32, y0: u32, x1: u32, y1: u32, color: u16) {
    if x1 > x0 {
      self.line(x0,y0,x1,y0,color);
    } else {
      self.line(x1,y0,x0,y0,color);
    }

    if y1 > y0 {
      self.line(x0,y0,x0,y1,color);
    } else {
      self.line(x0,y1,x0,y0,color);
    }

    if x1 > x0 {
      self.line(x0,y1,x1,y1,color);
    } else {
      self.line(x1,y1,x0,y1,color);
    }

    if y1 > y0 {
      self.line(x1,y0,x1,y1,color);
    } else {
      self.line(x1,y1,x1,y0,color);
    }
  }

  /// Draws a filled rectangle.
  fn fillrect(&self, x0_b: u32, y0_b: u32, x1_b: u32, y1_b: u32, color: u16) {
    let mut l: u32;
    let mut c: u32;
    let mut i: u32;
    let mut x0: u32 = x0_b;
    let mut y0: u32 = y0_b;
    let mut x1: u32 = x1_b;
    let mut y1: u32 = y1_b;
    if x0 > x1 {
      i = x0;
      x0 = x1;
      x1 = i;
    }

    if y0 > y1 {
      i = y0;
      y0 = y1;
      y1 = i;
    }

    l = x0;
    while l <= x1 {
      c = y0;
      while c <= y1 {
        self.pixel(l, c, color);
        c += 1;
      }
      l += 1;
    }
  }

  /// Draws an image from a buffer.
  fn image(&self, width: u32, height: u32, data: &[u16]) {
    for x in range(0, width) {
      for y in range(0, height) {
        self.pixel(x, y, data[(x+y*width) as uint]);
      }
    }
  }
}

#[cfg(test)]
mod test {
  use core::mem::zeroed;
  use core::option::{Some, None};
  use core::iter::{Iterator, Range, range};
  use core::cell::Cell;

  use drivers::chario::CharIO;
  use drivers::lcd::LCD;

  pub struct TestLCD {
    pixbuf: [[Cell<u16>, ..16], ..16],
  }

  impl CharIO for TestLCD {
    fn putc(&self, _: char) { }
  }

  impl LCD for TestLCD {
    fn flush(&self) { }

    fn clear(&self) { self.set_fill(0); }

    fn pixel(&self, x: u32, y: u32, color: u16) {
      if x >= 16 || y >= 16 {
        return
      }

      self.pixbuf[x as uint][y as uint].set(color);
    }
  }

  impl TestLCD {
    fn new() -> TestLCD {
      TestLCD {
        pixbuf: unsafe { zeroed() },
      }
    }

    fn coords(&self, x: uint, y: uint) -> (u32, u32) { (x as u32, y as u32) }

    fn axis(&self) -> Range<uint> { range(0u, 16) }

    fn for_each(&self, block: |(u32, u32), u16|) {
      for x in self.axis() {
        for y in self.axis() {
          block(self.coords(x, y), self.pixbuf[x][y].get());
        }
      }
    }

    fn map_each(&self, block: |(u32, u32), u16| -> u16) {
      for x in self.axis() {
        for y in self.axis() {
          self.pixbuf[x][y].set(block(self.coords(x, y), self.pixbuf[x][y].get()));
        }
      }
    }

    fn set_fill(&self, color: u16) {
      self.map_each(|_, _| { color });
    }
  }

  /* keep this
  let blank: [[u16, ..16], ..16] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
  ];
  */

  #[test]
  fn should_fill_and_clear() {
    let io = TestLCD::new();
    io.set_fill(128);
    io.for_each(|_, x| assert!(x == 128));
    io.clear();
    io.for_each(|_, x| assert!(x == 0));
  }

  #[test]
  fn should_set_pixels() {
    let io = TestLCD::new();
    io.map_each(|(x, y), _| { (x+y) as u16 });
    io.for_each(|(x, y), v| assert!(v == (x+y) as u16));
  }

  #[test]
  fn should_draw_line() {
    let io = TestLCD::new();

    let diagonal: [[u16, ..16], ..16] = [
      [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    ];

    io.line(0, 0, 15, 15, 1);

    // TODO(errordeveloper): investigate why this hangs the test run in `__psynch_cvwait()`
    // io.line(15, 15, 0, 0, 1);

    io.for_each(|(x, y), v| {
      assert!(v == diagonal[y as uint][x as uint]);
      assert!(v == diagonal[x as uint][y as uint]);
    });

    io.clear();

    let non_symetric: [[u16, ..16], ..16] = [
      [2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,3,3,3,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,5,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,5,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,5,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,4],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,4],
    ];

    io.line(0, 0, 0, 1, 2);
    io.line(4, 2, 4, 4, 3);
    io.line(14, 14, 14, 15, 4);
    io.line(15, 14, 15, 15, 4);
    io.line(11, 11, 13, 13, 5);

    io.for_each(|(x, y), v| {
      assert!(v == non_symetric[x as uint][y as uint]);
    });
  }

  #[test]
  fn should_draw_rect() {
    let io = TestLCD::new();

    let overlapping: [[u16, ..16], ..16] = [
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,6,6,6,6,6,6,0,0,0,0,0,0,0,0],
      [0,0,6,0,0,0,0,6,0,0,0,0,0,0,0,0],
      [0,0,6,0,0,0,0,6,0,0,0,0,0,0,0,0],
      [0,0,6,0,7,7,7,7,7,7,7,0,0,0,0,0],
      [0,0,6,0,7,0,0,6,0,0,7,0,0,0,0,0],
      [0,0,6,0,7,0,0,6,0,0,7,0,0,0,0,0],
      [0,0,6,0,7,7,7,7,7,7,7,0,0,0,0,0],
      [0,0,6,0,0,0,0,6,0,0,0,0,0,0,0,0],
      [0,0,6,6,6,6,6,6,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];

    assert!(overlapping[4][2] == 6);
    assert!(overlapping[12][7] == 6);
    assert!(overlapping[7][4] == 7);
    assert!(overlapping[10][10] == 7);

    io.rect(4, 2, 12, 7, 6);
    io.rect(7, 4, 10, 10, 7);

    io.for_each(|(x, y), v| {
      assert!(v == overlapping[x as uint][y as uint]);
    });
  }

  #[test]
  fn should_draw_fillrect() {
    let io = TestLCD::new();

    let eights: [[u16, ..16], ..16] = [
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,8,8,8,8,8,8,8,8,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];

    io.fillrect(4, 4, 11, 11, 8);

    io.for_each(|(x, y), v| {
      assert!(v == eights[x as uint][y as uint]);
    });
  }

  #[test]
  fn should_draw_image() {
    let io = TestLCD::new();

    let i1 = &[
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
      0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
      0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff,
    ];

    let i2: [[u16, ..16], ..16] = [
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
      [0xff, 0xff, 0xff, 0xad, 0xde, 0x10, 0x01, 0xde,
       0xed, 0x10, 0x01, 0xed, 0xad, 0xff, 0xff, 0xff],
    ];

    io.image(16, 16, i1);

    io.for_each(|(y, x), v| {
      assert!(v == i2[x as uint][y as uint]);
    });
  }
}
