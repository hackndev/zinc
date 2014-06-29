#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;
#[phase(plugin)] extern crate macro_platformtree;

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
)

#[no_split_stack]
fn run(args: &pt::run_args) {
  use zinc::hal::pin::GPIO;
  use zinc::hal::timer::Timer;

  args.led1.set_high();
  args.led2.set_low();
  args.timer.wait(1);

  args.led1.set_low();
  args.led2.set_high();
  args.timer.wait(1);
}
