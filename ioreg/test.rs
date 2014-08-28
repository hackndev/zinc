// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

//! Tests for ioreg! syntax extension

#![feature(phase)]
#[phase(plugin)] extern crate macro_ioreg;
#[phase(plugin,link)] extern crate shiny;
extern crate core;


#[allow(dead_code)]
#[path="../src/lib/volatile_cell.rs"] mod volatile_cell;

#[cfg(test)]
mod test {
  use std::mem::{transmute, zeroed};
  use std::ptr::RawPtr;
  use volatile_cell::VolatileCell;

  fn get_value<'a, T>(v: &'a T, offset: uint) -> u32 {
    unsafe {
      let ptr: *const u32 = transmute(v);
      *(ptr.offset(offset as int))
    }
  }

  fn zeroed_safe<T: Copy>() -> T {
    unsafe {
      return zeroed();
    }
  }

  ioregs!(BASIC_TEST = {
    0x0 => reg32 reg1 {
      0      => field1,
      1..3   => field2,
      16..24 => field3,
      25     => field4: set_to_clear,
    }
    0x4 => reg32 reg2 {
      0      => field1,
    }
  })
  
  describe!(
    before_each {
      let test: BASIC_TEST = zeroed_safe();
    }

    it "can round_trip simple field values 1" {
      test.reg1.set_field1(true);
      assert_eq!(test.reg1.field1(), true)
      assert_eq!(get_value(&test, 0), 1)
      assert_eq!(get_value(&test, 1), 0)
    }

    it "can round trip simple field values 2" {
      test.reg1.set_field3(0xde);
      assert_eq!(test.reg1.field3(), 0xde)
      assert_eq!(get_value(&test, 0), 0xde<<16)
    }

    it "sets set_to_clear fields" {
      test.reg1.clear_field4();
      assert_eq!(get_value(&test, 0), 1<<25)
    }
  )

  ioregs!(GROUP_TEST = {
    0x0 => group regs[5] {
      0x0 => reg32 reg1 {
        0..31 => field1
      }
      0x4 => reg32 reg2 {
        0..31 => field2
      }
    }
  })

  describe!(
    before_each {
      let test: GROUP_TEST = zeroed_safe();
    }

    it "sets groups correctly" {
      test.regs[0].reg1.set_field1(0xdeadbeef);
      assert_eq!(test.regs[0].reg1.field1(), 0xdeadbeef)
      assert_eq!(get_value(&test, 0), 0xdeadbeef)
      for i in range(1, 10) {
        assert_eq!(get_value(&test, i), 0)
      }

      test.regs[2].reg2.set_field2(0xfeedbeef);
      assert_eq!(test.regs[2].reg2.field2(), 0xfeedbeef)
      assert_eq!(get_value(&test, 5), 0xfeedbeef)
    }
  )

  ioregs!(FIELD_ARRAY_TEST = {
    0x0 => reg32 reg1 {
      0..31 => field[16]
    }
  })

  describe!(
    before_each {
      let test: FIELD_ARRAY_TEST = zeroed_safe();
    }

    it "sets field arrays correctly" {
      test.reg1.set_field(0, 1);
      assert_eq!(test.reg1.field(0), 1);
      assert_eq!(get_value(&test, 0), 0x1)

      test.reg1.set_field(4, 3);
      assert_eq!(test.reg1.field(4), 3);
      assert_eq!(get_value(&test, 0), 0x1 | 0x3<<8)
    }
  )
}
