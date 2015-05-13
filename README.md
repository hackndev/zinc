[![Build Status](https://travis-ci.org/hackndev/zinc.svg)](https://travis-ci.org/hackndev/zinc) [![Stories in Ready](https://badge.waffle.io/hackndev/zinc.png?label=ready&title=Ready)](https://waffle.io/hackndev/zinc)
Zinc, the bare metal stack for rust
===================================

## About zinc.rs

Zinc is an experimental attempt to write an ARM stack that would be similar to CMSIS or mbed in capabilities but would show rust's best safety features applied to embedded development.

Zinc is mostly assembly-free and completely C-free at the moment. One of the goals of zinc is to figure out, how much of the usual RTOS stack is it possible to write in rust in a safe manner, while keeping the resource usage profile low enough (comparable to C/C++ code).

Useful links:

 * [blog](http://zinc.rs/blog)
 * [build stats](http://zinc.rs/stats) (output binary size change over time)
 * [api docs](http://zinc.rs/apidocs/zinc)

## Main features

Zinc provides you with *safe* code in terms of rust code safety; accessing hardware directly is *unsafe*, but you can do that as well if you want.

In addition to *software safety*, zinc provides *hardware safety* with Platform Tree specification; you define the hardware configuration right in the code in simple key-value DSL and compiler verifies that all hardware is configured properly; that also allows to optimize the code to much bigger extent than with conventional RTOSes.

## Supported hardware

Zinc supports only ARM at the moment. The primary development is focused on two test boards with NXP LPC1768 and ST STM32F407. Other MCUs will follow when core API is stabilized.

## License

Zinc is distributed under Apache-2.0, see LICENSE for more details.

## Current Status

Zinc is currently undergoing some large build system changes to keep up with the latest developments in rust. Some things, most notably `platformtree` are not currently building, so note that some examples may not build. All non-platformtree examples should work fine though.

## Usage

Get a gcc cross-toolchain for arm and configure the configuration section of `Makefile` with information about it.

To build an application from examples/ use the following rake command:

```
PLATFORM=<platform> EXAMPLE_NAME=<appname> make build
```

Ouput will go to `target/<target-triple>/release/examples`.
