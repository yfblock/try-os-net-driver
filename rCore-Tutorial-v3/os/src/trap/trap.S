.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret

.altmacro
.set    CONTEXT_SIZE, 34
.macro SAVE_K reg, offset
    sd  \reg, \offset*8(sp)
.endm
.macro SAVE_K_N n
    SAVE_K  x\n, \n
.endm
.macro LOAD_K reg, offset
    ld  \reg, \offset*8(sp)
.endm
.macro LOAD_K_N n
    LOAD_K  x\n, \n
.endm


.global __kernel_trap
.section .text.trap
.align 2
__kernel_trap:
    addi    sp, sp, CONTEXT_SIZE*-8
    SAVE_K    x1, 1
    addi    x1, sp, 34*8
    SAVE_K    x1, 2
    .set    n, 3
    .rept   29
        SAVE_K_N  %n
        .set    n, n + 1
    .endr
    csrr    t0, sstatus
    csrr    t1, sepc
    SAVE_K    t0, 32
    SAVE_K    t1, 33
    add a0, x0, sp
    csrr a1, scause
    csrr a2, stval
    call trap_from_kernel
    LOAD_K    s1, 32
    LOAD_K    s2, 33
    csrw    sstatus, s1
    csrw    sepc, s2
    LOAD_K    x1, 1
    .set    n, 3
    .rept   29
        LOAD_K_N  %n
        .set    n, n + 1
    .endr
    LOAD_K    x2, 2
    sret