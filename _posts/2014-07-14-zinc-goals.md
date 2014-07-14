---
layout: post
title:  Zinc goals redefined and extended
---

The scope of zinc is now slowly getting stabilised. Currently zinc is a platform that allows (would allow) one to build a basic application based on the same basic principles that Arduino uses. Zinc provides developer with access to MCU peripherals and a bunch of drivers for common hardware found in hobbyist projects.

### How is zinc different from Arduino?

Rust allows to write **much** safer code than the usual arduino sketch. It means less problems in the runtime. You cannot bootstrap zinc with a simple `setup()` and `loop()` though, in a foreseeable future all supported platforms will be ported to platformtree, which means that a hardware specification must be written for an application. That one, while adding some typing to the "hello world" style application makes sure that hardware constraints are met at compile time (i.e. one cannot simply share same pin with a led and UART transmit and toggle it in different code paths, which makes sense for most of the cases).

### How is zinc different from mbed?

Zinc is actually closer to mbed rather than Arduino now. It shares the same architecture (ARM being the only one zinc supports for now). Zinc tends to be more lightweight on the runtime side, as all the hardware-specific checks are done at compile time, lots of runtime code can be saved (there's no reason to have code that computes UART clocks based on baud rate and system clock if the platform definition implies that system clock and baud rates will be constant).

Due to this, most of libzinc APIs, while maturing from *experimental* would be tagged as `unsafe`, that is — no runtime checks are done so it's developer's responsibility to make sure the arguments are valid. Those APIs are mostly for platform tree generator code, though, where PT parser can verify that all the arguments are in safe ranges.

So, getting back to zinc goals. The vision of zinc platform priorities is:

 * safe, rather than unsafe: all non-`unsafe` APIs are really safe to use, which is verified at compile time if at all possible;
 * fast, where applicable: all `unsafe` APIs skip any runtime checks and are highly optimised to do the job;
 * small: this is not currently true, as there's no optimisation for size in rust, but moving code from run time into compile time allows for smaller binary and memory footprint.
