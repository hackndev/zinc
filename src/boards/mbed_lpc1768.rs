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

/*
// TODO(farcaller): why can't I re-export those?
pub use pin5  = hal::lpc17xx::pin::map::port0::pin9;
pub use pin6  = hal::lpc17xx::pin::map::port0::pin8;
pub use pin7  = hal::lpc17xx::pin::map::port0::pin7;
pub use pin8  = hal::lpc17xx::pin::map::port0::pin6;
pub use pin9  = hal::lpc17xx::pin::map::port0::pin0;
pub use pin10 = hal::lpc17xx::pin::map::port0::pin1;
pub use pin11 = hal::lpc17xx::pin::map::port0::pin18;
pub use pin12 = hal::lpc17xx::pin::map::port0::pin17;
pub use pin13 = hal::lpc17xx::pin::map::port0::pin15;
pub use pin14 = hal::lpc17xx::pin::map::port0::pin16;
pub use pin15 = hal::lpc17xx::pin::map::port0::pin23;
pub use pin16 = hal::lpc17xx::pin::map::port0::pin24;
pub use pin17 = hal::lpc17xx::pin::map::port0::pin25;
pub use pin18 = hal::lpc17xx::pin::map::port0::pin26;

pub use pin19 = hal::lpc17xx::pin::map::port1::pin30;
pub use pin20 = hal::lpc17xx::pin::map::port1::pin31;

pub use pin21 = hal::lpc17xx::pin::map::port2::pin5;
pub use pin22 = hal::lpc17xx::pin::map::port2::pin4;
pub use pin23 = hal::lpc17xx::pin::map::port2::pin3;
pub use pin24 = hal::lpc17xx::pin::map::port2::pin2;
pub use pin25 = hal::lpc17xx::pin::map::port2::pin1;
pub use pin26 = hal::lpc17xx::pin::map::port2::pin0;

pub use pin27 = hal::lpc17xx::pin::map::port0::pin11;
pub use pin28 = hal::lpc17xx::pin::map::port0::pin10;
pub use pin29 = hal::lpc17xx::pin::map::port0::pin5;
pub use pin30 = hal::lpc17xx::pin::map::port0::pin4;
*/

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
