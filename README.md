[![Build Status](https://travis-ci.org/hackndev/zinc.svg)](https://travis-ci.org/hackndev/zinc)
[![irc](https://img.shields.io/badge/irc-%23zinc-lightgrey.svg)](https://kiwiirc.com/client/irc.mozilla.org?chan=#zinc)
[![Stories in Ready](https://badge.waffle.io/hackndev/zinc.png?label=ready&title=Ready)](https://waffle.io/hackndev/zinc)
Zinc, the bare metal stack for rust
===================================

## About zinc.rs

Zinc is an experimental attempt to write an ARM stack that would be
similar to CMSIS or mbed in capabilities but would show rust's best
safety features applied to embedded development.

Zinc is mostly assembly-free and completely C-free at the moment. One
of the goals of zinc is to figure out, how much of the usual RTOS
stack is it possible to write in rust in a safe manner, while keeping
the resource usage profile low enough (comparable to C/C++ code).

Useful links:

 * [blog](http://zinc.rs/blog)
 * [build stats](http://zinc.rs/stats) (output binary size change over
   time)
 * [api docs](http://zinc.rs/apidocs/zinc)

## Main features

Zinc provides you with *safe* code in terms of rust code safety;
accessing hardware directly is *unsafe*, but you can do that as well
if you want.

In addition to *software safety*, zinc provides *hardware safety* with
Platform Tree specification; you define the hardware configuration
right in the code in simple key-value DSL and compiler verifies that
all hardware is configured properly; that also allows to optimize the
code to much bigger extent than with conventional RTOSes.

There is also an effort to generate platform tree specifications for
new platforms from
[ARM CMSIS SVDs](http://www.keil.com/pack/doc/CMSIS/SVD/html/index.html)
such as those aggregated by the
[cmsis-svd](http://www.keil.com/pack/doc/CMSIS/SVD/html/index.html)
project.

## Supported hardware

Some level of support is currently provided for the following
processor families.  Not all peripherals are supported for all
processors at this time.

* NXP lpc11xx
* NXP lpc17xx
* Freescale k20
* ST STM32f4
* ST STM32l1

In the future, a better story will be available that will allow for
additional processor families, processors, and boards using those
processors to be defined more easily.

## License

Zinc is distributed under Apache-2.0, see LICENSE for more details.

## Usage

### Environment Setup

For the time being, Zinc makes use unstable features of the Rust
language.  As such, we recommend using the latest nightly version of
Rust for development.  As features of the language stabilize over
time, it is the goal of Zinc to eventually be able to target stable
versions of the compiler.
[rustup.rs](https://www.rustup.rs/) may be used to manage
installations of multiple versions of rust on single machine.

The currently supported Rust version is nightly-2016-09-17.  To install
it with rustup use the following:

```Shell
rustup install nightly-2016-09-17
rustup override set nightly-2016-09-17
```

In addition to rust itself, a GCC cross-toolchain for ARM must be
installed.  Although LLVM is used for a majority of compilation, the
GCC Linker is still used at this time.  The
[GCCM ARM Embedded toolchain](https://launchpad.net/gcc-arm-embedded/+download)
works well for most people.

### Building Examples Within Zinc

There are several examples available within the Zinc source itself.  Zinc makes
use of Xargo (a bare metal variant of Cargo) for its build system, but it is
still necesary to provide the build system with a few pieces of information for
it to properly compile for your target.

Namely, xargo must know about and have access to:

1. The target specification for the machine being specified (consumed
   by the compiler)
2. A feature telling the code what platform is being targetted.  These
   features are defined in the form `mcu_<platform>`.

Suppose we are targetting the `k20` platform.  In that case, I could
build the `blink_k20` example program by doing the following.  Refer
to [build-jenkins.sh](support/build-jenkins.sh) for a mapping of
platforms to targets.

```
$ cd examples/blink_k20
$ ln -s ../../thumbv7em-none-eabi.json
$ xargo build --target=thumbv7em-none-eabi --features mcu_k20 --release

$ file target/thumbv7em-none-eabi/release/blink
target/thumbv7em-none-eabi/release/blink: ELF 32-bit LSB executable, ARM, EABI5 version 1 (SYSV), statically linked, not stripped
```

If you receive link errors, you probably need to tell Xargo to use
your cross-compilers linker.  You can do this by adding a
`.xargo/config` to either your home directory or the root of the Zinc
project:

```toml
[target.thumbv7em-none-eabi]
linker = "arm-none-eabi-gcc"
ar = "arm-none-eabi-ar"
```

### Using Zinc for your Project

Since Zinc uses xargo for its build system, using Zinc from your own
project just requires setting up your Cargo.toml correctly.

You can find an example of how to do that here:
https://github.com/posborne/zinc-example-lpc1768

### Contacting developers

You can find us at IRC: [#zinc](https://kiwiirc.com/client/irc.mozilla.org?chan=#zinc) on irc.mozilla.org.
