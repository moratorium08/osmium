.option norvc
.section .reset.boot, "ax",@progbits
.global _start
.global abort

_start:
    lui     sp, %hi(stack_end)
    addi    sp, sp, %lo(stack_end)
    j       __start_rust

stack:
    .skip 4096
stack_end:
