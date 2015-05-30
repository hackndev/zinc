__STACK_BASE  = 0x10002000;

INCLUDE iomem.ld

isr_reserved_1 = 0 - (__STACK_BASE + main + 1 + isr_nmi + 1 + isr_hardfault + 1);

_data_load = LOADADDR(.data);

ENTRY(main)

MEMORY
{
    rom(RX)   : ORIGIN = 0x00000000, LENGTH = 0x8000
    ram(WAIL) : ORIGIN = 0x10000000, LENGTH = 0x2000
}

REGION_ALIAS("vectors", rom);

INCLUDE layout_common.ld
