#![feature(plugin, no_std, core)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

use zinc::drivers::chario::CharIO;
use zinc::drivers::lcd::hd44780u::{Hd44780u, Font};

platformtree!(
  tiva_c@mcu {
    clock {
      source = "MOSC";
      xtal   = "X16_0MHz";
      pll    = false;
    }

    gpio {
      a {
        d7@5 { direction = "out"; }
      }
      b {
        rs@0 { direction = "out"; }
        en@1 { direction = "out"; }
        d6@4 { direction = "out"; }
      }
      e {
        d4@4 { direction = "out"; }
        d5@5 { direction = "out"; }
      }
    }

    timer {
      /* The mcu contain both 16/32bit and "wide" 32/64bit timers. */
      timer@w0 {
        /* prescale sysclk to 1Mhz since the wait code expects 1us
         * granularity */
        prescale = 80;
        mode = "periodic";
      }
    }
  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
        rs   = &rs;
        en   = &en;
        d4   = &d4;
        d5   = &d5;
        d6   = &d6;
        d7   = &d7;
      }
    }
  }
);


pub fn run(args: &pt::run_args) {

  let lcd = Hd44780u::new(args.timer,
                          args.rs,
                          args.en,
                          [ args.d4, args.d5, args.d6, args.d7 ]);

  lcd.init(true, Font::Font5x8);

  // Create custom 'heart' character at index 0.
  lcd.custom_char_5x8(0,
                      [0b00000,
                       0b01010,
                       0b11111,
                       0b11111,
                       0b11111,
                       0b01110,
                       0b00100,
                       0b00000]);
  // Create custom 'stick figure' character at index 1
  lcd.custom_char_5x8(1,
                      [0b00100,
                       0b01010,
                       0b00100,
                       0b11111,
                       0b00100,
                       0b01010,
                       0b10001,
                       0b00000]);

  // Enable blinking
  lcd.display_control(true, false, true);

  // Display a message surounded by two hearts
  lcd.puts("\0  Hello Zinc  \0");

  // Move to the 2nd line
  lcd.set_pos(0, 1);

  // Write a line of stick figures
  lcd.puts("\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01");

  // Move the cursor back to the middle of the first line
  lcd.set_pos(8, 0);

  loop {}
}
