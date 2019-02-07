OUTPUT_ARCH("riscv")

ENTRY(_start)

SECTIONS
{
    . = 0x00000000;

    .reset.boot :
    {
	*(.boot)
    }

    .text :
    {
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rdata .rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss bss.*)
    }


    . = ALIGN( 0x1000 );
    .kernel_aligned :
    {
        *(.kernel_aligned kernel_aligned.*)
    }
    
    PROVIDE(kernel_end = .);
}