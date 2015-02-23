#![feature(plugin, no_std, core)]
#![crate_type="staticlib"]
#![no_std]
#![plugin(macro_platformtree)]

extern crate core;
extern crate zinc;
#[macro_use] #[no_link] extern crate macro_platformtree;

use core::option::Option::{Some, None};

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
        dht_pin@4;
      }
    }
  }

  drivers {
    dht@dht22 {
      pin = &dht_pin;
      timer = &timer;
    }
  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
        uart = &uart;
        dht = &dht;
      }
    }
  }
);

#[zinc_task]
fn run(args: &pt::run_args) {
  use zinc::drivers::chario::CharIO;
  use zinc::hal::timer::Timer;

  args.timer.wait(3);

  let ret = args.dht.read();
  match ret {
    Some(v) => {
      args.uart.puts("temp:     "); args.uart.puti(v.temperature as u32);
      args.uart.puts("\n");
      args.uart.puts("humidity: "); args.uart.puti(v.humidity as u32);
      args.uart.puts("\n");
    },
    None => {
      args.uart.puts("fail\n");
    },
  }
}
