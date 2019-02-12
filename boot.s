.option norvc
.section .reset.boot, "ax",@progbits
.global _start
.global abort

_start:
    /* Set up stack pointer. */
    lui     sp, %hi(stack_end)
    addi    sp, sp, %lo(stack_end)
    /* Now jump to the rust world; __start_rust.  */
    j       __start_rust

.option norvc
.section .kernel_aligned, "ax",@progbits
.global kernel_pgdir_ptr
kernel_pgdir_ptr:
    .skip 4096
.global kernel_frames_ptr
kernel_frames_ptr:
    .skip 2097152
stack:
    .skip 4097152
stack_end: