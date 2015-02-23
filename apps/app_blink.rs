#![feature(plugin, no_std, core)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

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
      1 {
        led1@18 { direction = "out"; }
        led2@20 { direction = "out"; }
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

fn run(args: &pt::run_args) {
  use zinc::hal::pin::Gpio;
  use zinc::hal::timer::Timer;

  args.led1.set_high();
  args.led2.set_low();
  args.timer.wait(1);

  args.led1.set_low();
  args.led2.set_high();
  args.timer.wait(1);
}
