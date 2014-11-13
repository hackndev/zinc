__STACK_BASE = 0x20008000;  /* end of ram */
_data_load = LOADADDR(.data);

INCLUDE ./src/zinc/hal/stm32l1/iomem.ld

ENTRY(main)

MEMORY
{
    rom(RX)     : ORIGIN = 0x08000000, LENGTH = 6 * 64K
    ram(WAIL)   : ORIGIN = 0x20000000, LENGTH = 32K
}

REGION_ALIAS("vectors", rom);

INCLUDE ./src/zinc/hal/layout_common.ld
