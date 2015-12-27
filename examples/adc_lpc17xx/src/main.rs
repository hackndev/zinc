// ADC example for lpc17xx chips
// Copyright 2015 Felix Obenhuber <felix@obenhuber.de>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(start, plugin, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;

use zinc::hal::pin::{Adc, Gpio};
use zinc::hal::timer::Timer;

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
            0 {
                led@22 { direction = "out"; }
                adc0@23 { direction = "out"; function = "ad0_0"; }
            }
        }
    }

    os {
        single_task {
            loop = "run";
            args {
                adc0 = &adc0;
                led = &led;
                timer = &timer;
            }
        }
    }
);

fn run(args: &pt::run_args) {
    loop {
        if args.adc0.read() > 2048 {
            args.led.set_high();
        } else {
            args.led.set_low();
        }
        args.timer.wait_ms(100);
    }
}

