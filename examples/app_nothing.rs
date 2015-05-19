#![feature(no_std, core, start)]
#![no_std]

extern crate zinc;

#[start]
fn start(_: isize, _: *const *const u8) -> isize {
    main();
    0
}

pub fn main() {
}
