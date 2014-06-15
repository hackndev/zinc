#![feature(asm)]
#![crate_type="staticlib"]
#![no_std]
#![feature(phase)]

extern crate zinc;
#[phase(plugin)] extern crate macro_platformtree;

platformtree!(
  mcu@lpc17xx {
    clock {
      source = "main-oscillator";
      source_frequency = 12_000_000;
      pll {
        m = 50;
        n = 3;
        divisor = 4;
      }
    }
  }

  // os@os {
  //   single_task {
  //     loop = "run";
  //   }
  // }
)

#[no_split_stack]
fn run() {
  loop {
    unsafe { asm!("nop") }
  }
}
