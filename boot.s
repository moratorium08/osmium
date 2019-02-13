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

.section .elfdata
.incbin "misc/bin/nop"

.option norvc
.section .kernel_aligned, "ax",@progbits
.global kernel_pgdir_ptr
kernel_pgdir_ptr:
    .skip 4096
.global temporary_pgdir_ptr
temporary_pgdir_ptr:
    .skip 4096

.global kernel_frames_ptr
kernel_frames_ptr:
    .skip 2097152

.global nop_start
nop_start:
    .incbin "misc/bin/nop"
.global nop_end
nop_end:
stack:
    .skip 8097152
stack_end:

