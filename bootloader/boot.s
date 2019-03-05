.option norvc
.section .reset.boot, "ax",@progbits
.global _start
.global abort

_start:
    lui     a0, %hi(boot_time_trap_handler)
    addi    a0, a0, %lo(boot_time_trap_handler)
    slli    a0, a0, 2
    csrrs   x0, stvec, a0
    
    lui     sp, %hi(stack_end)
    addi    sp, sp, %lo(stack_end)
    j       __start_rust

.option norvc
.section .stack, "ax",@progbits
stack:
    .skip 4096
stack_end:
