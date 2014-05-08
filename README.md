[![Build Status](https://travis-ci.org/hackndev/zinc.svg)](https://travis-ci.org/hackndev/zinc) [![Stories in Ready](https://badge.waffle.io/hackndev/zinc.png?label=ready&title=Ready)](https://waffle.io/hackndev/zinc)
Zinc, the bare metal stack for rust
===================================

Zinc is an experimental attempt to write an ARM stack, that would be similar to
CMSIS in capabilities but would show rust's best safety features applied to
embedded development.

Zinc is mostly assembly-free and completely C-free at the moment. One of the
goals of zinc is to figure out, how much of the usual RTOS stack is it possible
to write in rust in a safe manner, while keeping the resource usage profile low
enough (comparable to C/C++ code).

## Supported hardware

Zinc supports *only* ARM at the moment, but any architecture supported by LLVM
should work. There might be some plans to port zin over to other architectures
some time in the future.

Currently supported ARM MCUs:

 * NXP LPC1768 based on mbed board — good support;
 * ST STM32F407 based on STM32F4Discovery board — draft support.

## Usage

Get a gcc cross-toolchain for arm and configure `TOOLCHAIN` and
`TOOLCHAIN_LIBS_PATH` in Rakefile header as appropriate.

To build an application from apps/ use the following rake command:

```
rake PLATFORM=<platform> build_all  # or build_<appname>
```

## License

Zinc is distributed under Apache-2.0, see LICENSE for more details.
