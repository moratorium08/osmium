.option norvc
.section .reset.boot, "ax",@progbits
.global _start
.global abort

_start:
    /* Set up stack pointer. */
    li     sp, 0xf4230
    /* Now jump to the rust world; __start_rust.  */
    j       __start_rust


.bss

stacks:
    .skip 1024

.option norvc
.section .kernel_aligned, "ax",@progbits
.global kernel_pgdir_ptr
kernel_pgdir_ptr:
    .skip 4096