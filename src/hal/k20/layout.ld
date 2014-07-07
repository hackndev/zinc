/* FIXME(bgamari): Make stack base configurable? */
__STACK_BASE  = 0x20000000;

_boot_checksum = 0 - (__STACK_BASE + main + 1 + isr_nmi + 1 + isr_hardfault + 1);

_data_load = LOADADDR(.data);

INCLUDE ./src/hal/k20/iomem.ld

ENTRY(main)

/* For MK20DX128 */
MEMORY
{
    rom(RX)   : ORIGIN = 0, LENGTH = 128K
    ram(WAIL) : ORIGIN = 0x20000000 - 16K / 2, LENGTH = 16K
}

INCLUDE ./src/hal/layout_common.ld
