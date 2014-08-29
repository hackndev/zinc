__STACK_BASE  = 0x10002000;

_boot_checksum = 0 - (__STACK_BASE + main + 1 + isr_nmi + 1 + isr_hardfault + 1);

_data_load = LOADADDR(.data);

INCLUDE ./src/zinc/hal/lpc17xx/iomem.ld

ENTRY(main)

MEMORY
{
    rom(RX)   : ORIGIN = 0x00000000, LENGTH = 0x8000
    ram(WAIL) : ORIGIN = 0x10000000, LENGTH = 0x2000
}

REGION_ALIAS("vectors", rom);

INCLUDE ./src/zinc/hal/layout_common.ld
