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

use drivers::chario::CharIO;

#[cfg(cfg_mcu_has_spi)] pub mod c12332;
#[cfg(cfg_mcu_has_spi, FIXME_BROKEN)] pub mod ili9341;
pub mod font_small_7;

pub trait LCD : CharIO {
  fn clear(&self);
  fn flush(&self);
  fn pixel(&self, x: i32, y: i32, color: u16);
  #[allow(dead_assignment)]
  fn line(&self, x0_b: i32, y0_b: i32, x1: i32, y1: i32, color: u16) {
    let mut x0: i32 = x0_b;
    let mut y0: i32 = y0_b;
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;
    let mut dx_sym: i32 = 0;
    let mut dy_sym: i32 = 0;
    let mut dx_x2: i32 = 0;
    let mut dy_x2: i32 = 0;
    let mut di = 0;

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
      di = dy_x2 - dx;
      while x0 != x1 {
        self.pixel(x0, y0, color);
        x0 += dx_sym;
        if di < 0 {
          di += dy_x2;
        } else {
          di += dy_x2 - dx_x2;
          y0 += dy_sym;
        }
      }
      self.pixel(x0, y0, color);
    } else {
      di = dx_x2 - dy;
      while y0 != y1 {
        self.pixel(x0, y0, color);
        y0 += dy_sym;
        if di < 0 {
          di += dx_x2;
        } else {
          di += dx_x2 - dy_x2;
          x0 += dx_sym;
        }
      }
      self.pixel(x0, y0, color);
    }
  }

  fn rect(&self, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
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

  fn fillrect(&self, x0_b: i32, y0_b: i32, x1_b: i32, y1_b: i32, color: u16) {
    let mut l: i32;
    let mut c: i32;
    let mut i: i32;
    let mut x0: i32 = x0_b;
    let mut y0: i32 = y0_b;
    let mut x1: i32 = x1_b;
    let mut y1: i32 = y1_b;
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

  fn draw_image(&self, width: u32, height: u32, data: &[u16]) {
    let mut x: u32 = 0;
    let mut y: u32;

    while x < width {
      y = 0;
      while y < height {
        self.pixel(x as i32, y as i32, data[(x+y*width) as uint]);
        y += 1;
      }
      x += 1;
    }
  }
}

#[allow(dead_code)]
enum Colors {
  ColorRed = 0xf800u16,
  ColorGreen = 0x07e0u16,
  ColorBlue = 0x001fu16,
}
