__STACK_BASE = 0x20004FFF;  /* end of ram */
_data_load = LOADADDR(.data);

INCLUDE iomem.ld

ENTRY(main)

MEMORY
{
    rom(RX)     : ORIGIN = 0x08000000, LENGTH = 64K
    ram(WAIL)   : ORIGIN = 0x20000000, LENGTH = 20K
}

REGION_ALIAS("vectors", rom);

INCLUDE layout_common.ld
