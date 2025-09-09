ENTRY(boot)

SECTIONS {
    . = 0x80200000;

    /* Kernel text section */
    .text : {
        KEEP(*(.text.boot)); /* Keep boot function at the beginning */
        *(.text .text.*);
        . = ALIGN(4);
        *(.text.stvec);
        . = ALIGN(4);
    }

    /* Read-only data */
    .rodata : ALIGN(4) {
        *(.rodata .rodata.*);
    }

    /* Initialized data */
    .data : ALIGN(4) {
        *(.data .data.*);
    }

    /* Uninitialized data */
    .bss : ALIGN(4) {
        __bss_start = .;
        *(.bss .bss.* .sbss .sbss.*);
        __bss_end = .;
    }

    /* Kernel heap area */
    . = ALIGN(4096); /* Page align */
    __heap_start = .;
    . += 1024 * 1024; /* 1MB heap */
    __heap_end = .;

    /* Kernel stack */
    . = ALIGN(16);
    . += 64 * 1024; /* 64KB stack */
    __stack_top = .;

    /* End of kernel */
    __kernel_end = .;

    /DISCARD/ : {
        *(.eh_frame);
        *(.comment);
    }
}