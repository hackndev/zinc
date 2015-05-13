_boot_checksum = 0; /* TODO(farcaller): extract this to lpc code only */
_data_load = LOADADDR(.data);

ENTRY(main)

MEMORY
{
    rom(RX)   : ORIGIN = 0x00000000, LENGTH = 0x40000
    ram(WAIL) : ORIGIN = 0x20000000, LENGTH = 0x8000 /* 32KB SRAM */
}

__STACK_BASE  = ORIGIN(ram) + LENGTH(ram);

REGION_ALIAS("vectors", rom);

INCLUDE ./src/hal/layout_common.ld
