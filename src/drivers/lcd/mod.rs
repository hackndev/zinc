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

use core::iter::range_inclusive;

use drivers::chario::CharIO;

pub mod c12332;
pub mod ili9341;
pub mod font_small_7;
pub mod hd44780u;

/// LCD provides a generic interface to a TFT LCD peripheral.
///
/// It provides generic methods for drawing primitives and bitmaps based on
/// `pixel` to set a pixel.
///
/// LCD does not flush buffers automatically, user must call `flush` after the
/// drawing sequence to actually display the data on screen.
pub trait LCD : CharIO {
  /// Clears the screen.
  fn clear(&self);

  /// Flushes the internal buffer to screen, where applicable.
  fn flush(&self);

  /// Sets one pixel color. The actual color bits are driver-specific.
  fn pixel(&self, x: u32, y: u32, color: u16);

  /// Draws a line from xy0 to xy1.
  fn line(&self, x0_b: u32, y0_b: u32, x1: u32, y1: u32, color: u16) {
    let (mut x0, mut y0) = (x0_b, y0_b);

    let (dx, dy) = ((x1-x0) as i32, (y1-y0) as i32);

    let dx_sym: i32 =
      if dx > 0 {
        1
      } else {
        -1
      };

    let dy_sym: i32 =
      if dy > 0 {
        1
      } else {
        -1
      };

    let (dx, dy) = (dx_sym*dx, dy_sym*dy);

    let (dx_x2, dy_x2) = (dx*2, dy*2);

    if dx >= dy {
      let mut di = dy_x2 - dx;
      while x0 != x1 {
        self.pixel(x0, y0, color);
        x0 += dx_sym as u32;
        if di < 0 {
          di += dy_x2;
        } else {
          di += dy_x2 - dx_x2;
          y0 += dy_sym as u32;
        }
      }
      self.pixel(x0, y0, color);
    } else {
      let mut di = dx_x2 - dy;
      while y0 != y1 {
        self.pixel(x0, y0, color);
        y0 += dy_sym as u32;
        if di < 0 {
          di += dx_x2;
        } else {
          di += dx_x2 - dy_x2;
          x0 += dx_sym as u32;
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

    let (x0, x1) =
      if x0_b > x1_b {
        (x1_b, x0_b)
      } else {
        (x0_b, x1_b)
      };

    let (y0, y1) =
      if y0_b > y1_b {
        (y1_b, y0_b)
      } else {
        (y0_b, y1_b)
      };

    for l in range_inclusive(x0, x1) {
      for c in range_inclusive(y0, y1) {
        self.pixel(l as u32, c as u32, color);
      }
    }
  }

  /// Draws an image from a buffer.
  fn image(&self, width: u32, height: u32, data: &[u16]) {
    for x in 0..width {
      for y in 0..height {
        self.pixel(x, y, data[(x+y*width) as usize]);
      }
    }
  }
}

#[cfg(test)]
mod test {
  use core::mem::zeroed;
  use core::cell::Cell;
  use core::ops::Fn;

  use drivers::chario::CharIO;
  use drivers::lcd::LCD;

  pub struct TestLCD {
    pixbuf: [[Cell<u16>; 16]; 16],
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

      self.pixbuf[x as usize][y as usize].set(color);
    }
  }

  impl TestLCD {
    fn new() -> TestLCD {
      TestLCD {
        pixbuf: unsafe { zeroed() },
      }
    }

    fn coords(&self, x: usize, y: usize) -> (u32, u32) { (x as u32, y as u32) }

    fn axis(&self) -> Range<usize> { 0..16 }

    fn for_each<F>(&self, block: F) where F: Fn((u32, u32), u16) {
      for x in self.axis() {
        for y in self.axis() {
          block(self.coords(x, y), self.pixbuf[x][y].get());
        }
      }
    }

    fn map_each<F>(&self, block: F) where F: Fn((u32, u32), u16) -> u16 {
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
  let blank: [[u16; 16]; 16] = [
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

    let diagonal: [[u16; 16]; 16] = [
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

    io.line(15, 15, 0, 0, 1);

    io.for_each(|(x, y), v| {
      assert!(v == diagonal[y as usize][x as usize]);
      assert!(v == diagonal[x as usize][y as usize]);
    });

    io.clear();

    let non_symetric: [[u16; 16]; 16] = [
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
      assert!(v == non_symetric[x as usize][y as usize]);
    });
  }

  #[test]
  fn should_draw_rect() {
    let io = TestLCD::new();

    let overlapping: [[u16; 16]; 16] = [
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
    io.rect(10, 10, 7, 4, 7);

    io.for_each(|(x, y), v| {
      assert!(v == overlapping[x as usize][y as usize]);
    });
  }

  #[test]
  fn should_draw_fillrect() {
    let io = TestLCD::new();

    let eights: [[u16; 16]; 16] = [
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
      assert!(v == eights[x as usize][y as usize]);
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

    let i2: [[u16; 16]; 16] = [
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
      assert!(v == i2[x as usize][y as usize]);
    });
  }
}
