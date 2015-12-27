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

#![feature(plugin)]
#![plugin(ioreg)]

extern crate volatile_cell;
extern crate core;

#[cfg(test)]
mod test {
  use std::mem::{transmute, zeroed};
  use volatile_cell::VolatileCell;

  fn get_value<'a, T>(v: &'a T, offset: usize) -> u32 {
    unsafe {
      let ptr: *const u32 = transmute(v);
      *(ptr.offset(offset as isize))
    }
  }

  fn zeroed_safe<T: Copy>() -> T {
    unsafe {
      return zeroed();
    }
  }

  ioregs!(BASIC_TEST @ 0 = {
    0x0 => reg32 reg1 {
      0      => field1,
      1..3   => field2,
      16..24 => field3,
      25     => field4: set_to_clear,
    }
    0x4 => reg32 reg2 {
      0      => field1,
    }
    0x8 => reg32 wo_reg {
      0..15  => field1: wo,
      16..31 => field2: wo,
    }
  });

  #[test]
  fn round_trip_simple_field_values_1() {
    let test: BASIC_TEST = zeroed_safe();

    test.reg1.set_field1(true);
    assert_eq!(test.reg1.field1(), true);
    assert_eq!(get_value(&test, 0), 1);
    assert_eq!(get_value(&test, 1), 0);
  }

  #[test]
  fn round_trip_simple_field_values_2() {
    let test: BASIC_TEST = zeroed_safe();

    test.reg1.set_field3(0xde);
    assert_eq!(test.reg1.field3(), 0xde);
    assert_eq!(get_value(&test, 0), 0xde<<16);
  }

  #[test]
  fn set_set_to_clear_fields() {
    let test: BASIC_TEST = zeroed_safe();

    test.reg1.clear_field4();
    assert_eq!(get_value(&test, 0), 1<<25);
  }

  #[test]
  fn no_read_writeonly_registers() {
    let test: BASIC_TEST = zeroed_safe();

    test.wo_reg.set_field1(0xdead);
    assert_eq!(get_value(&test, 2), 0xdead);
    test.wo_reg.set_field2(0xdead);
    assert_eq!(get_value(&test, 2), 0xdead<<16);
  }

  #[test]
  fn set_field_twice() {
    let test: BASIC_TEST = zeroed_safe();
    
    test.reg1.set_field3(0xAA).set_field3(0x55);
    assert_eq!(test.reg1.field3(), 0x55);
  }

  #[test]
  fn set_independent_fields() {
    let test: BASIC_TEST = zeroed_safe();
    
    test.reg1.set_field2(0b010).set_field3(0xf0);
    
    assert_eq!(test.reg1.field2(), 0b010);
    assert_eq!(test.reg1.field3(), 0xf0);
  }

  /*
     describe!(
     before_each {
     let test: BASIC_TEST = zeroed_safe();
     }

     it "can round_trip simple field values 1" {
     test.reg1.set_field1(true);
     assert_eq!(test.reg1.field1(), true);
     assert_eq!(get_value(&test, 0), 1);
     assert_eq!(get_value(&test, 1), 0);
     }

     it "can round trip simple field values 2" {
     test.reg1.set_field3(0xde);
     assert_eq!(test.reg1.field3(), 0xde);
     assert_eq!(get_value(&test, 0), 0xde<<16);
     }

     it "sets set_to_clear fields" {
     test.reg1.clear_field4();
     assert_eq!(get_value(&test, 0), 1<<25);
     }

     it "does not read from writeonly registers" {
     test.wo_reg.set_field1(0xdead);
     assert_eq!(get_value(&test, 2), 0xdead);
     test.wo_reg.set_field2(0xdead);
     assert_eq!(get_value(&test, 2), 0xdead<<16);
     }
     );
     */

  ioregs!(GROUP_TEST @ 0 = {
    0x0 => group regs[5] {
      0x0 => reg32 reg1 {
        0..31 => field1
      }
      0x4 => reg32 reg2 {
        0..31 => field2
      }
    }
  });

  #[test]
  fn sets_groups_correctly() {
    let test: GROUP_TEST = zeroed_safe();
    test.regs[0].reg1.set_field1(0xdeadbeef);
    assert_eq!(test.regs[0].reg1.field1(), 0xdeadbeef);
    assert_eq!(get_value(&test, 0), 0xdeadbeef);
    for i in 1..10 {
      assert_eq!(get_value(&test, i), 0);
    }

    test.regs[2].reg2.set_field2(0xfeedbeef);
    assert_eq!(test.regs[2].reg2.field2(), 0xfeedbeef);
    assert_eq!(get_value(&test, 5), 0xfeedbeef);
  }

  /*
     describe!(
     before_each {
     let test: GROUP_TEST = zeroed_safe();
     }

     it "sets groups correctly" {
     test.regs[0].reg1.set_field1(0xdeadbeef);
     assert_eq!(test.regs[0].reg1.field1(), 0xdeadbeef);
     assert_eq!(get_value(&test, 0), 0xdeadbeef);
     for i in range(1, 10) {
     assert_eq!(get_value(&test, i), 0);
     }

     test.regs[2].reg2.set_field2(0xfeedbeef);
     assert_eq!(test.regs[2].reg2.field2(), 0xfeedbeef);
     assert_eq!(get_value(&test, 5), 0xfeedbeef);
     }
     );
     */

  ioregs!(FIELD_ARRAY_TEST @ 0 = {
    0x0 => reg32 reg1 {
      0..31 => field[16]
    }
  });

  #[test]
  fn sets_field_arrays_correctly() {
    let test: FIELD_ARRAY_TEST = zeroed_safe();
    test.reg1.set_field(0, 1);
    assert_eq!(test.reg1.field(0), 1);
    assert_eq!(get_value(&test, 0), 0x1);

    test.reg1.set_field(4, 3);
    assert_eq!(test.reg1.field(4), 3);
    assert_eq!(get_value(&test, 0), 0x1 | 0x3<<8);
  }

  /*
     describe!(
     before_each {
     let test: FIELD_ARRAY_TEST = zeroed_safe();
     }

     it "sets field arrays correctly" {
     test.reg1.set_field(0, 1);
     assert_eq!(test.reg1.field(0), 1);
     assert_eq!(get_value(&test, 0), 0x1);

     test.reg1.set_field(4, 3);
     assert_eq!(test.reg1.field(4), 3);
     assert_eq!(get_value(&test, 0), 0x1 | 0x3<<8);
     }
     );
     */

  ioregs!(GAP_TEST @ 0 = {
    0x0 => reg32 reg1 {
      0..31 => field,
    }
    0x10 => reg32 reg2 {
      0..31 => field,
    }
    0x14 => reg32 reg3 {
      0..31 => field,
    }
    0x20 => reg32 reg4 {
      0..31 => field,
    }
  });

  #[test]
  fn has_zero_base_offset() {
    let test: GAP_TEST = zeroed_safe();
    let base = &test as *const GAP_TEST;
    let addr = &test.reg1 as *const GAP_TEST_reg1;
    assert_eq!(addr as usize - base as usize, 0x0);
  }

  #[test]
  fn computes_first_gap() {
    let test: GAP_TEST = zeroed_safe();
    let base = &test as *const GAP_TEST;
    let addr = &test.reg2 as *const GAP_TEST_reg2;
    assert_eq!(addr as usize - base as usize, 0x10);
  }

  #[test]
  fn computes_second_gap() {
    let test: GAP_TEST = zeroed_safe();
    let base = &test as *const GAP_TEST;
    let addr = &test.reg4 as *const GAP_TEST_reg4;
    assert_eq!(addr as usize - base as usize, 0x20);
  }

  /*
     describe!(
     before_each {
     let test: GAP_TEST = zeroed_safe();
     let base = &test as *const GAP_TEST;
     }
     it "has zero base offset" {
     let addr = &test.reg1 as *const GAP_TEST_reg1;
     assert_eq!(addr as usize - base as usize, 0x0);
     }
     it "computes the correct first gap" {
     let addr = &test.reg2 as *const GAP_TEST_reg2;
     assert_eq!(addr as usize - base as usize, 0x10);
     }
     it "computes the correct second gap" {
     let addr = &test.reg4 as *const GAP_TEST_reg4;
     assert_eq!(addr as usize - base as usize, 0x20);
     }
     );
     */

  ioregs!(MULTI_TEST @ 0 = {
    0x100 => reg32 reg1[8] {
      0..31 => field[32],
    }
  });

  #[test]
  fn multi_ok() {
    let test: MULTI_TEST = zeroed_safe();
    test.reg1[0].set_field(0, true);
    assert_eq!(test.reg1[0].field(0), true);
  }
}
