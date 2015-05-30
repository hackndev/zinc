#![feature(core, plugin, asm, no_std)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

platformtree!(
  lpc17xx@mcu {
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

  os {
    single_task {
      loop = "run";
    }
  }
);

fn run() {
  unsafe { asm!("nop") }
}
