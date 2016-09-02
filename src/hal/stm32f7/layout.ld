__STACK_BASE = 0x2004FFFF;
_data_load = LOADADDR(.data);

INCLUDE iomem.ld

ENTRY(main)

MEMORY
{
    rom(RX)     : ORIGIN = 0x08000000, LENGTH = 1024K
    ram_c(WAIL) : ORIGIN = 0x10000000, LENGTH = 64K

    /* DTCM:  0x20000000 for  64K
     * SRAM1: 0x20010000 for 240K
     * SRAM2: 0x2004C000 for  16K
     */
    ram(WAIL)   : ORIGIN = 0x20000000, LENGTH = 64K + 240K + 16K
}

REGION_ALIAS("vectors", rom);

INCLUDE layout_common.ld
