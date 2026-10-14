ENTRY(kmain)
SECTIONS {
    . = 0x10000;
    .text   : { KEEP(*(.text.entry)) *(.text .text.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data   : { *(.data .data.*) }
    .bss    : { __bss_start = .; *(.bss .bss.*) *(COMMON) __bss_end = .; }

    /* Unwind tables are useless with panic = "abort". If kept, the linker
       may place them at 0x10000, ahead of kmain. */
    /DISCARD/ : { *(.eh_frame) *(.eh_frame_hdr) *(.comment) *(.note .note.*) }
}