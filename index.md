---
layout: bootstrap
menuidx: index
---

## About zinc.rs

Zinc is an experimental attempt to write an ARM stack that would be similar to CMSIS or mbed in capabilities but would show rust's best safety features applied to embedded development.

Zinc is mostly assembly-free and completely C-free at the moment. One of the goals of zinc is to figure out, how much of the usual RTOS stack is it possible to write in rust in a safe manner, while keeping the resource usage profile low enough (comparable to C/C++ code).

## Main features

Zinc provides you with *safe* code in terms of rust code safety; accessing hardware directly is *unsafe*, but you can do that as well if you want.

In addition to *software safety*, zinc provides *hardware safety* with Platform Tree specification; you define the hardware configuration right in the code in simple key-value DSL and compiler verifies that all hardware is configured properly; that also allows to optimize the code to much bigger extent than with conventional RTOSes.

## Supported hardware

Zinc supports only ARM at the moment. The primary development is focused on two test boards with NXP LPC1768 and ST STM32F407. Other MCUs will follow when core API is stabilized.

## License

Zinc is distributed under Apache-2.0, see LICENSE for more details.

## Blink demo code with comments

{% highlight rust %}
#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

// you can only use libcore features with zinc
extern crate core;
extern crate zinc;
#[phase(plugin)] extern crate macro_platformtree;

// this is the platform tree definition, a combination of hardware and software
// specification that gets verified at compile time and compiles down to
// optimised initialization blocks
platformtree!(
  // use lpc17xx
  lpc17xx@mcu {
    clock {
      // source is clocked from main (external) oscillator running at 12MHz
      source = "main-oscillator";
      source_frequency = 12_000_000;
      // configure pll to output 100MHz
      pll {
        m = 50;
        n = 3;
        divisor = 4;
      }
    }

    timer {
      // define configuration for timer 1
      timer@1 {
        counter = 25;
        divisor = 4;
      }
    }

    gpio {
      // define pins for gpio port 1
      1 {
        // pins 18 and 20 are gpio out
        led1@18 { direction = "out"; }
        led2@20 { direction = "out"; }
      }
    }
  }

  os {
    // define a single task (arduino-style, scheduler is not being compiled in)
    single_task {
      // a function to call in the loop
      loop = "run";
      args {
        // args are resolved to named nodes above and are passed into `run` as
        // pointers of the appropriate type
        timer = &timer;
        led1 = &led1;
        led2 = &led2;
      }
    }
  }
)

#[no_split_stack]
// this is the main function, it's only argument type is pt::run_args, a
// generated struct that references materialized nodes from platform tree
fn run(args: &pt::run_args) {
  // toggles pin values
  args.led1.set_high();
  args.led2.set_low();
  // wait for 1 second
  (args.timer as &zinc::hal::timer::Timer).wait(1);

  args.led1.set_low();
  args.led2.set_high();
  (args.timer as &zinc::hal::timer::Timer).wait(1);
}
{% endhighlight %}
