global long_mode_start

section .text
bits 64
long_mode_start:
    ; load 0 into all data segment registers
    ; needed so that some of the instructions won't read old protected mode segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; call the rust main
    extern _start
    call _start

    hlt
