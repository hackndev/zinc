_stack_base = 0x2001FFFF;
_boot_checksum = 0; /* TODO(farcaller): extract this to lpc code only */

_data_load = LOADADDR(.data);

ENTRY(main)

MEMORY
{
    rom(RX)     : ORIGIN = 0x08000000, LENGTH = 1024K
    ram_c(WAIL) : ORIGIN = 0x10000000, LENGTH = 64K
    ram(WAIL)   : ORIGIN = 0x20000000, LENGTH = 112K + 16K
    /* ram(WAIL) : ORIGIN = 0x2001C000, LENGTH = 16K */
}

SECTIONS
{
    .text : ALIGN(4)
    {
        FILL(0xff)

        KEEP(*(.isr_vector))
        KEEP(*(.isr_vector_nvic))
    } > rom

    .text : ALIGN(4)
    {
        FILL(0xff)

        *(.text*)
        *(.rodata .rodata.*)
    } > rom

    .ARM.exidx :
    {
        __exidx_start = .;
        *(.ARM.exidx* .gnu.linkonce.armexidx.*)
        __exidx_end = .;
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
    } > ram

    /DISCARD/ :
    {
        *(.glue_7*)  /* arm-thumb interworking */
        *(.v4_bx)  /* ARMv4 interworking fixup for missing BX */
        *(.vfp11_veneer)  /* VFP11 bugfixes s.a. http://sourceware.org/ml/binutils/2006-12/msg00196.html */
        *(.iplt .igot.plt)  /* STT_GNU_IFUNC symbols */
        *(.rel.*)  /* dynamic relocations */
    }
}
