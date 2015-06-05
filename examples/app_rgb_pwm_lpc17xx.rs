// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(plugin, no_std, core, start)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

use zinc::hal::timer::Timer;
use zinc::hal::lpc17xx::pwm;
use zinc::hal::pwm::PWMOutput;

// This example shows use of the RGB LED that is availble on the MBED
// Application Board.  The LED is connected to 3 pins coming
// from the MBED LPC1768.  Here's the mapping:
//
// - RGB_RED   => p23 => p2.3 (PWM1.4)
// - RGB_GREEN => p24 => p2.2 (PWM1.3)
// - RGB_BLUE  => p25 => p2.1 (PWM1.2)

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
        rgb_blue@3 {
          direction = "out";
          function = "pwm1_4";
        }
        rgb_green@2 {
          direction = "out";
          function = "pwm1_3";
        }
        rgb_red@1 {
          direction = "out";
          function = "pwm1_2";
        }
      }
    }
  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
        rgb_red = &rgb_red;
        rgb_green = &rgb_green;
        rgb_blue = &rgb_blue;
      }
    }
  }
);

fn do_color(timer: &Timer, pwm: &mut pwm::PWM) {
  for i in 0..100 {
    pwm.write(i as f32 / 100.0f32);
    timer.wait_ms(10);
  }
}

fn run(args: &pt::run_args) {
  let mut pwm_red = pwm::PWM::new(pwm::PWMChannel::CHANNEL3);
  let mut pwm_green = pwm::PWM::new(pwm::PWMChannel::CHANNEL2);
  let mut pwm_blue = pwm::PWM::new(pwm::PWMChannel::CHANNEL1);

  pwm_red.set_period_us(10_000);
  pwm_green.set_period_us(10_000);
  pwm_blue.set_period_us(10_000);

  loop {
    pwm_red.write(1.0);
    pwm_green.write(1.0);
    pwm_blue.write(1.0);
  }
}
