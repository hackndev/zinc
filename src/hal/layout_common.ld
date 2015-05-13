SECTIONS
{
    .vector : ALIGN(4)
    {
        FILL(0xff)

        KEEP(*(.isr_vector))
        KEEP(*(.isr_vector_nvic))
    } > vectors

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
