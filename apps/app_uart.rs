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

    uart {
      uart@0 {
        baud_rate = 115200;
        mode = "8N1";
        tx = &uart_tx;
        rx = &uart_rx;
      }
    }

    gpio {
      0 {
        uart_tx@2;
        uart_rx@3;
      }
      1 {
        led4@23 { direction = "out"; }
      }
    }
  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
        txled = &led4;
        uart = &uart;
      }
    }
  }
);

fn run(args: &pt::run_args) {
  use zinc::drivers::chario::CharIO;
  use zinc::hal::timer::Timer;
  use zinc::hal::pin::Gpio;

  args.uart.puts("Hello, world\n");

  let mut i = 0;
  loop {
    args.txled.set_high();
    args.uart.puts("Waiting for ");
    args.uart.puti(i);
    args.uart.puts(" seconds...\n");

    i += 1;
    args.txled.set_low();

    args.timer.wait(1);
  }
}
