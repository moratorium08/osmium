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
.incbin "../misc/bin/nop"

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
    .incbin "../misc/bin/nop"
.global nop_end
nop_end:
.global loop_start
loop_start:
    .incbin "../misc/bin/loop"
.global loop_end
loop_end:
.global sh_start
sh_start:
    .incbin "../misc/bin/sh"
.global sh_end
sh_end:
.global syscaller_start
syscaller_start:
    .incbin "../misc/bin/syscaller"
.global syscaller_end
syscaller_end:
.global hello_start
hello_start:
    .incbin "../misc/bin/hello"
.global hello_end
hello_end:
.global tic_start
tic_start:
    .incbin "../misc/bin/tic"
.global tic_end
tic_end:
.global catch_ball_start
catch_ball_start:
    .incbin "../misc/bin/catch_ball"
.global catch_ball_end
catch_ball_end:
