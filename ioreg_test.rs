#![feature(phase)]
extern crate core;
#[phase(plugin)] extern crate macro_ioreg;

use core::kinds::marker;
use core::intrinsics::{volatile_load, volatile_store};

use std::io::File;
use std::mem;
use std::c_vec::CVec;

// TODO(bharrisau) I don't know enough about markers - is it better
// to just use an Unsafe<T> here instead?
pub struct VolatileCell<T> {
  value: T,
  invariant: marker::InvariantType<T>,
}

impl<T> VolatileCell<T> {
  pub fn new(value: T) -> VolatileCell<T> {
    VolatileCell {
      value: value,
      invariant: marker::InvariantType::<T>,
    }
  }

  #[inline]
  pub fn get(&self) -> T {
    unsafe {
      volatile_load(&self.value)
    }
  }

  #[inline]
  pub fn set(&self, value: T) {
    unsafe {
      volatile_store(&self.value as *const T as *mut T, value)
    }
  }
}

ioregs!(
    group FTM {
        0x0  => SC: u32  /// Status and control register
        {
             0..2 => PS:     uint       /// Prescale
             3..4 => CLKS:   enum { NO_CLOCK=0x0, SYSTEM_CLOCK=0x1, FIXED_FREQ=0x2, EXTERNAL=0x3, }
             5    => CPWMS:  bool
             6    => TOIE:   bool
             7    => TOF:    ro bool
        }
    
        0x4  => CNT: u32 /// Count register
        {
            0..15 => COUNT:  uint
        }
    
        0x8  => MOD: u32 /// Modulo register
        {
            0..15 => MOD:    uint
        }
    
        group Channel {
            0x0 => CSC:    u32         /// Compare/capture channel status and control register
            {
                0 => DMA:    bool
                2 => ELSA:   bool
                3 => ELSB:   bool
                4 => MSA:    bool
                5 => MSB:    bool
                6 => CHIE:   bool
                7 => CHF:    bool
            }
    
            0x4 => CV:     u32
            {
                0..15 => VAL:    uint
            }
        }
    
        0xc  =>      CHANNELS: Channel[8]   /// Compare/capture channels
    
        0x4c =>      CNTIN: u32             /// Counter initial value register
        {
            0..15 => INIT:     uint
        }
    
        0x50 =>      STATUS: u32            /// Channel status register
        {
            0..7 => CHF:      bool[8]
        }
        0x60 =>      TEST: u32              /// This is only a test
        {
            0..7 => TEST:     bool[8]
        }
    }
)

pub fn main() {
  unsafe {
    let len = 0x60;
    let ftm: FTM = std::mem::zeroed();
    ftm.MOD.set_MOD(0xdead);
    ftm.CHANNELS[0].CSC.set_DMA(true);

    let vec: CVec<u8> = CVec::new(mem::transmute(&ftm), len);
    let path = Path::new("reg");
    let mut f = File::create(&path);
    match f.write(vec.as_slice()) {
      Ok(_) => {},
      Err(e) => println!("Error: {}", e),
    }
  }
}
