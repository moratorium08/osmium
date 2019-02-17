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

# (for reserve region. use for page copy between two different memory space)
.global tmp_reserved_page
tmp_reserved_page: 
    .skip 4096

.global kernel_frames_ptr
kernel_frames_ptr:
    .skip 2097152

.global stack_stop
stack_stop:
    .skip 4096
stack:
    .skip 4096000
stack_end:

.global interrupt_stack_stop
interrupt_stack_stop:
    .skip 4096
interrupt_stack:
    .skip 4096000
.global interrupt_stack_end
interrupt_stack_end:

# followings are not aligned data
.option norvc
.section .programs, "ax",@progbits
.global nop_start
nop_start:
    .incbin "misc/bin/syscaller"
.global nop_end
nop_end:

