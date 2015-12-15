// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(plugin, start, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

use zinc::hal::timer::Timer;
use zinc::hal::lpc17xx::pwm;
use zinc::hal::pwm::PWMOutput;

// This example shows use of the RGB LED that is availble on the MBED
// Application Board.  The LED is connected to 3 pins coming
// from the MBED LPC1768.  Here's the mapping:
//
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

    timer {
      timer@1 {
        counter = 25;
        divisor = 4;
      }
    }

    gpio {
      2 {
        // LPC1768 DIPP25 - P2.1/PWM1.2/RXD1
        rgb_blue@1 {
          direction = "out";
          function = "pwm1_2";
        }
        // LPC1768 DIPP24 - P2.2/PWM1.3/TRACEDATA3
        rgb_green@2 {
          direction = "out";
          function = "pwm1_3";
        }
        // LPC1768 DIPP23 - P2.3/PWM1.4/TRACEDATA2
        rgb_red@3 {
          direction = "out";
          function = "pwm1_4";
        }
      }
    }
  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
      }
    }
  }
);

fn run(args: &pt::run_args) {
  let mut pwm_red = pwm::PWM::new(pwm::PWMChannel::Channel4, 20_000);
  let mut pwm_green = pwm::PWM::new(pwm::PWMChannel::Channel3, 20_000);
  let mut pwm_blue = pwm::PWM::new(pwm::PWMChannel::Channel2, 20_000);

  // turn all off
  pwm_red.write(0.0);
  pwm_green.write(1.0);
  pwm_blue.write(1.0);

  loop {
    for pwm in &mut [pwm_red, pwm_green, pwm_blue] {
      for i in 1..100 {
        pwm.write((i as f32) / 100.0);
        args.timer.wait_ms(10);
      }
      pwm.write(1.0); // turn off channel
    }
  }
}
