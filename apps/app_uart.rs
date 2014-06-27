#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;
#[phase(plugin)] extern crate macro_platformtree;

platformtree_verbose!(
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
)

#[no_split_stack]
fn run(args: &pt::run_args) {
  let uart = args.uart as &zinc::drivers::chario::CharIO;
  let timer = args.timer as &zinc::hal::timer::Timer;

  uart.puts("Hello, world\n");

  let mut i = 0;
  loop {
    args.txled.set_high();
    uart.puts("Waiting for ");
    uart.puti(i);
    uart.puts(" seconds...\n");

    i += 1;
    args.txled.set_low();

    timer.wait(1);
  }
}
