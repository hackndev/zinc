__STACK_BASE = 0x2001FFFF;
_data_load = LOADADDR(.data);

INCLUDE ./src/hal/stm32f4/iomem.ld

ENTRY(main)

MEMORY
{
    rom(RX)     : ORIGIN = 0x08000000, LENGTH = 1024K
    ram_c(WAIL) : ORIGIN = 0x10000000, LENGTH = 64K
    ram(WAIL)   : ORIGIN = 0x20000000, LENGTH = 112K + 16K
    /* ram(WAIL) : ORIGIN = 0x2001C000, LENGTH = 16K */
}

REGION_ALIAS("vectors", rom);

INCLUDE ./src/hal/layout_common.ld
