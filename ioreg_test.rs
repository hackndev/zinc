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
    FTM = {
        /// FlexTimer module.
        ///
        /// This is a timer module
        0x0  => reg32 SC  /// Status and control register
        {
             0..2 => PS,           /// Prescale
             3..4 => CLKS
             {
                 0x0 => NO_CLOCK,     /// no clock selected
                 0x1 => SYSTEM_CLOCK, /// use system clock
                 0x2 => FIXED_FREQ,   /// use fixed frequency clock
                 0x3 => EXTERNAL,     /// use external clock
             },
             5    => CPWMS,
             6    => TOIE,
             7    => TOF: ro,
        },
    
        0x4  => reg32 CNT /// Count register
        {
            0..15 => COUNT,
        },
    
        0x8  => reg32 MOD /// Modulo register
        {
            0..15 => MOD,
        },
    
        0xc  => group CH[8]                 /// Compare/capture channels
        {
            0x0 => reg32 CSC                /// Compare/capture channel status and control register
            {
                0 => DMA,
                2 => ELSA,
                3 => ELSB,
                4 => MSA,
                5 => MSB,
                6 => CHIE,
                7 => CHF,
            },
    
            0x4 => reg32 CV                 /// Channel counter value
            {
                0..15 => VAL,
            },
        },
    
        0x4c => reg32 CNTIN                 /// Counter initial value register
        {
            0..15 => INIT,
        },
    
        0x50 => reg32 STATUS                /// Channel status register
        {
            0..7 => CHF[8],
        },
    }
)

pub fn main() {
  unsafe {
    let len = 0x60;
    let ftm: FTM = std::mem::zeroed();
    ftm.MOD.set_MOD(0xdead);
    ftm.CH[0].CSC.set_DMA(true);

    let vec: CVec<u8> = CVec::new(mem::transmute(&ftm), len);
    let path = Path::new("reg");
    let mut f = File::create(&path);
    match f.write(vec.as_slice()) {
      Ok(_) => {},
      Err(e) => println!("Error: {}", e),
    }
  }
}
