/* FIXME(bgamari): Make stack base configurable? */
__STACK_BASE  = 0x20001FFF;

_data_load = LOADADDR(.data);

INCLUDE iomem.ld

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
REGION_ALIAS("flash_config", FCFG)
REGION_ALIAS("rom", FLASH);
REGION_ALIAS("ram", RAM);

SECTIONS
{
    .vector : ALIGN(4)
    {
        FILL(0xff)

        KEEP(*(.isr_vector))
        KEEP(*(.isr_vector_nvic))
    } > vectors

    .flashcfg : ALIGN(4)
    {
        KEEP(*(.flash_configuration))
    } > flash_config

    .text : ALIGN(4)
    {
        FILL(0xff)
        *(.text*)
        *(.rodata .rodata.*)
    } > rom

    .data : ALIGN(4)
    {
        _data = .;

        *(SORT_BY_ALIGNMENT(.data*))
        . = ALIGN(4);

        _edata = .;
    } > ram AT>rom = 0xff

    .bss : ALIGN(4)
    {
        _bss = .;

        *(.bss*)
        *(COMMON)
        . = ALIGN(4);

        _ebss = .;

        . += 4;

        __STACK_LIMIT = .;

        . += 4;

        _eglobals = .;
    } > ram

    /DISCARD/ :
    {
        *(.glue_7*)  /* arm-thumb interworking */
        *(.v4_bx)  /* ARMv4 interworking fixup for missing BX */
        *(.vfp11_veneer)  /* VFP11 bugfixes s.a. http://sourceware.org/ml/binutils/2006-12/msg00196.html */
        *(.iplt .igot.plt)  /* STT_GNU_IFUNC symbols */
        *(.rel.*)  /* dynamic relocations */
        *(.ARM.exidx*) /* index entries for section unwinding */
        *(.ARM.extab*) /* exception unwinding information */
    }
}

