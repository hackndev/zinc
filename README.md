[![Stories in Ready](https://badge.waffle.io/hackndev/zinc.png?label=ready&title=Ready)](https://waffle.io/hackndev/zinc)
Zinc, the bare metal stack for rust
===================================

Zinc is an experimental attempt to write an ARM stack, that would be similar to
CMSIS in capabilities but would show rust's best safety features applied to
embedded development.

## Usage

Get a gcc cross-toolchain for arm and configure `TOOLCHAIN` and
`TOOLCHAIN_LIBS_PATH` as appropriate.

To build an application from apps/ use the following rake command:

```
rake PLATFORM=<platform> APP=<app>
```

## The basic roadmap for zinc

 * Implement a clean hal interface for NXP LPC1768 based on mbed board;
 * Implement a clean hal interface for ST STM32F4 based on STM32F4Discovery;
 * Implement basic RTOS features, including processes with safe stacks;
 * Implement basic networking stack, most possibly UDPv6 over ethernet or 802.15.4.

## License

Zinc is distributed under apache-2.0, see LICENSE for more details.
