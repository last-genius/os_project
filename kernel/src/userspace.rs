#[naked]
pub unsafe fn prog1() {
    asm!("\
        mov rbx, 0xf0000000
        prog1start:
        push 0x0
        mov rbp, 0x0
        mov rax, 0x1
        mov rcx, 0x3
        mov rdx, 0x4
        mov rdi, 0x6
        mov r8, 0x7
        mov r9, 0x8
        mov r10, 0x9
        mov r11, 0x10
        mov r12, 0x11
        mov r13, 0x12
        mov r14, 0x13
        mov r15, 0x14
        xor rax, rax
        prog1loop:
        inc rax
        cmp rax, 0x4000000
        jnz prog1loop
        pop rax
        inc rbx
        mov rdi, rsp
        mov rsi, rbx
        syscall
        jmp prog1start
    ");
}

#[naked]
pub unsafe fn prog2() {
    asm!("\
        mov rbx, 0
        prog2start:
        push 0x1
        mov rbp, 0x100
        mov rax, 0x101
        mov rcx, 0x103
        mov rdx, 0x104
        mov rdi, 0x106
        mov r8, 0x107
        mov r9, 0x108
        mov r10, 0x109
        mov r11, 0x110
        mov r12, 0x111
        mov r13, 0x112
        mov r14, 0x113
        mov r15, 0x114
        xor rax, rax
        prog2loop:
        inc rax
        cmp rax, 0x4000000
        jnz prog2loop
        pop rax
        inc rbx
        mov rdi, rsp
        mov rsi, rbx
        syscall
        jmp prog2start
    ");
}
