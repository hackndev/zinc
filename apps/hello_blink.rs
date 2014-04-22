#![crate_type="staticlib"]
#![no_std]

extern crate zinc;

use zinc::hal;
use zinc::hal::gpio::Pin;

#[cfg(mcu_stm32f4)] use zinc::hal::stm32f4;
#[cfg(mcu_lpc17xx)] use zinc::hal::lpc17xx;

struct Platform {
  led1: Pin,
  led2: Pin,
}

#[cfg(mcu_stm32f4)]
static platform: Platform = Platform {
  led1: Pin {
    port:     stm32f4::gpio::PortD,
    pin:      12,
    function: stm32f4::gpio::GPIOOut,
  },
  led2: Pin {
    port:     stm32f4::gpio::PortD,
    pin:      14,
    function: stm32f4::gpio::GPIOOut,
  },
};

#[cfg(mcu_lpc17xx)]
static platform: Platform = Platform {
  led1: Pin {
    port:     lpc17xx::gpio::Port1,
    pin:      18,
    function: lpc17xx::gpio::GPIOOut,
  },
  led2: Pin {
    port:     lpc17xx::gpio::Port1,
    pin:      20,
    function: lpc17xx::gpio::GPIOOut,
  },
};

#[cfg(mcu_stm32f4)]
fn enable_port_clocking() {
  hal::stm32f4::gpio::enable_port_clock(platform.led1.port);
}

#[cfg(mcu_lpc17xx)]
fn enable_port_clocking() {}

#[no_split_stack]
#[no_mangle]
#[start]
pub extern fn main() {
  hal::platform_init();

  enable_port_clocking();

  let led1 = platform.led1.configure();
  let led2 = platform.led2.configure();

  loop {
    led1.set_high();
    led2.set_high();
    hal::timer::wait(1);
    led1.set_low();
    led2.set_low();
    hal::timer::wait(1);
  }
}
