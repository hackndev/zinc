#![feature(plugin, no_std, core)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

platformtree!(
  tiva_c@mcu {
    clock {
      source = "PIOSC";
      xtal   = "X16_0MHz";
      pll    = true;
      div    = 5;
    }

    gpio {
      f {
        led1@1 { direction = "out"; }
        led2@2 { direction = "out"; }
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
        led1 = &led1;
        led2 = &led2;
      }
    }
  }
);

pub fn run(args: &pt::run_args) {
  use zinc::hal::pin::Gpio;
  use zinc::hal::timer::Timer;

  loop {
    args.led1.set_high();
    args.led2.set_low();

    args.timer.wait(1);

    args.led1.set_low();
    args.led2.set_high();

    args.timer.wait(1);
  }
}
