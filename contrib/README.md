Zinc Contrib
============

This folder contains all the device specific files needed for building
binaries with Zinc.

How to use
----------

Copy the three files below into your project and make the appropriate
changes as required below.

Target definitions
------------------

The JSON files are target definitions for the various ARM cores supported by
Zinc.

- Cortex M0
- Cortex M0+
- Cortex M1
- Cortex M3
- Cortex M4
- Cortex M4F

Layout definitions
------------------

Linker scripts are stored in each vendor folder. These contain the layout of
flash, RAM, and special function addresses like the NVIC table.

Device specific module
----------------------

Rust modules for utilising device peripherals are provided.
