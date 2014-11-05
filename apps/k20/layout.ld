/* FIXME(bgamari): Make stack base configurable? */
__STACK_BASE  = 0x20001FFF;

_boot_checksum = 0;

_data_load = LOADADDR(.data);


ENTRY(main)

/* For MK20DX32 */
MEMORY
{
    VECT (R)      : ORIGIN = 0x00000000, LENGTH = 0x3FC  /* Vector area */
    FIRC (R)      : ORIGIN = 0x000003FC, LENGTH = 4      /* Custom IRC user trim */
    FCFG (R)      : ORIGIN = 0x00000400, LENGTH = 16     /* Flash config */
    FLASH (RX)    : ORIGIN = 0x00000410, LENGTH = 32K - 0x410
    RAM (WAIL)    : ORIGIN = 0x20000000 - 8K / 2, LENGTH = 8K
}

REGION_ALIAS("vectors", VECT);
REGION_ALIAS("rom", FLASH);
REGION_ALIAS("ram", RAM);

INCLUDE layout_common.ld
