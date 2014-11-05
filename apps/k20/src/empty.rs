#![feature(asm, linkage)]
#![no_std]
#![no_main]

extern crate core;
extern crate zinc;

pub mod mk20_dx256_vlh7;

#[no_mangle]
pub unsafe extern fn main() {
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();
  run();

  loop {};
}

fn run() {
  unsafe { asm!("nop") }
}

