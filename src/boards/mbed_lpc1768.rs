// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
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

/*!
Default configuration for mbed board: http://mbed.org/platforms/mbed-LPC1768

This configuraiton is clocked at 100MHz with Timer1 calibrated for 1us ticks.
It also provides GPIOConfs for all four leds found on the board.
*/

use hal::lpc17xx::init::{SysConf, Clock, Main, PLL0};
use hal::gpio::{GPIOConf, Out};
use hal::lpc17xx::pin::map;

pub static configuration: SysConf = SysConf {
  clock: Clock {
    source: Main(12_000_000),
    pll: PLL0 {
      enabled: true,
      m: 50,
      n: 3,
      divisor: 4,
    }
  },
};

pub static led1: GPIOConf = GPIOConf {
  pin: map::port1::pin18::GPIO,
  direction: Out,
};

pub static led2: GPIOConf = GPIOConf {
  pin: map::port1::pin20::GPIO,
  direction: Out,
};

pub static led3: GPIOConf = GPIOConf {
  pin: map::port1::pin21::GPIO,
  direction: Out,
};

pub static led4: GPIOConf = GPIOConf {
  pin: map::port1::pin23::GPIO,
  direction: Out,
};
