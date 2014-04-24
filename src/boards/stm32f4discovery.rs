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
Default configuration for STM32F4Discovery board

This configuraiton is clocked at 168MHz with Timer1 calibrated for 1us ticks.
It also provides GPIOConfs for all four leds found on the board.
*/

use hal::stm32f4::init::{SysConf, ClockConf};
use hal::stm32f4::init::{SystemClockPLL,PLLConf, PLLClockHSE};
use hal::stm32f4::pin;
use hal::gpio::{GPIOConf, Out};

pub static configuration: SysConf = SysConf {
  clock: ClockConf {
    source: SystemClockPLL(PLLConf {
      source: PLLClockHSE(8_000_000),
      m: 8,
      n: 336,
      p: 2,
      q: 7,
    })
  }
};

pub static led1: GPIOConf = GPIOConf {
  pin: pin::PinConf {
    port:     pin::PortD,
    pin:      12,
    function: pin::GPIOOut,
  },
  direction: Out,
};

pub static led2: GPIOConf = GPIOConf {
  pin: pin::PinConf {
    port:     pin::PortD,
    pin:      13,
    function: pin::GPIOOut,
  },
  direction: Out,
};

pub static led3: GPIOConf = GPIOConf {
  pin: pin::PinConf {
    port:     pin::PortD,
    pin:      14,
    function: pin::GPIOOut,
  },
  direction: Out,
};

pub static led4: GPIOConf = GPIOConf {
  pin: pin::PinConf {
    port:     pin::PortD,
    pin:      15,
    function: pin::GPIOOut,
  },
  direction: Out,
};
