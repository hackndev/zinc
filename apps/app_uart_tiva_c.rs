#![feature(plugin, no_std, core)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

use zinc::drivers::chario::CharIO;

platformtree!(
  tiva_c@mcu {
    clock {
      source  = "MOSC";
      xtal    = "X16_0MHz";
      pll     = true;
      div     = 5;
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

    gpio {
      a {
        uart_rx@0 {
          direction = "in";
          function  = 1;
        }
        uart_tx@1 {
          direction = "in";
          function  = 1;
        }
      }
      f {
        txled@2 { direction = "out"; }
      }
    }

    uart {
      uart@0 {
        mode = "115200,8n1";
      }
    }

  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
        uart = &uart;
        txled = &txled;
        uart_tx = &uart_tx;
      }
    }
  }
);

fn run(args: &pt::run_args) {
  use zinc::hal::timer::Timer;
  use zinc::hal::pin::Gpio;

  args.uart.puts("Hello, world\r\n");

  let mut i = 0;
  loop {
    args.txled.set_high();
    args.uart.puts("Waiting for ");
    args.uart.puti(i);
    args.uart.puts(" seconds...\r\n");

    i += 1;
    args.txled.set_low();

    args.timer.wait(1);
  }
}
