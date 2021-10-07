ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
        . = ALIGN(8);
        /* ensure that the multiboot header is at the beginning */
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text)
    }
}
