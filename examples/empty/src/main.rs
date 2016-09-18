#![feature(plugin, asm, start)]
#![no_std]
#![plugin(macro_zinc)]

extern crate zinc;

use zinc::hal::mem_init::{init_data, init_stack};

#[zinc_main]
fn run() {
  init_data();
  init_stack();
  unsafe { asm!("nop") }
}
