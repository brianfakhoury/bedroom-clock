# Byte loops for compiler-generated copies and fills. Nothing here moves enough
# memory to need the larger, faster routines that would otherwise be linked
.section .text.memory,"ax"
.global memcpy
memcpy:
    mv t0, a0
1:  beqz a2, 2f
    lbu t1, 0(a1)
    sb t1, 0(t0)
    addi a1, a1, 1
    addi t0, t0, 1
    addi a2, a2, -1
    j 1b
2:  ret

.global memset
memset:
    mv t0, a0
1:  beqz a2, 2f
    sb a1, 0(t0)
    addi t0, t0, 1
    addi a2, a2, -1
    j 1b
2:  ret
