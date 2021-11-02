#include "isr.h"
#include "idt.h"
#include "../drivers/screen.h"
#include "../kernel/util.h"

/* Can't do this with a loop because we need the address
 * of the function names */
void isr_install() {
    set_idt_gate(0, (u64)isr0);
    set_idt_gate(1, (u64)isr1);
    set_idt_gate(2, (u64)isr2);
    set_idt_gate(3, (u64)isr3);
    set_idt_gate(4, (u64)isr4);
    set_idt_gate(5, (u64)isr5);
    set_idt_gate(6, (u64)isr6);
    set_idt_gate(7, (u64)isr7);
    set_idt_gate(8, (u64)isr8);
    set_idt_gate(9, (u64)isr9);
    set_idt_gate(10, (u64)isr10);
    set_idt_gate(11, (u64)isr11);
    set_idt_gate(12, (u64)isr12);
    set_idt_gate(13, (u64)isr13);
    set_idt_gate(14, (u64)isr14);
    set_idt_gate(15, (u64)isr15);
    set_idt_gate(16, (u64)isr16);
    set_idt_gate(17, (u64)isr17);
    set_idt_gate(18, (u64)isr18);
    set_idt_gate(19, (u64)isr19);
    set_idt_gate(20, (u64)isr20);
    set_idt_gate(21, (u64)isr21);
    set_idt_gate(22, (u64)isr22);
    set_idt_gate(23, (u64)isr23);
    set_idt_gate(24, (u64)isr24);
    set_idt_gate(25, (u64)isr25);
    set_idt_gate(26, (u64)isr26);
    set_idt_gate(27, (u64)isr27);
    set_idt_gate(28, (u64)isr28);
    set_idt_gate(29, (u64)isr29);
    set_idt_gate(30, (u64)isr30);
    set_idt_gate(31, (u64)isr31);

    set_idt(); // Load with ASM
}

/* To print the message which defines every exception */
char *exception_messages[] = {
    "Division By Zero",
    "Debug",
    "Non Maskable Interrupt",
    "Breakpoint",
    "Into Detected Overflow",
    "Out of Bounds",
    "Invalid Opcode",
    "No Coprocessor",

    "Double Fault",
    "Coprocessor Segment Overrun",
    "Bad TSS",
    "Segment Not Present",
    "Stack Fault",
    "General Protection Fault",
    "Page Fault",
    "Unknown Interrupt",

    "Coprocessor Fault",
    "Alignment Check",
    "Machine Check",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",

    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved"
};

void isr_handler(registers_t r) {
    kprint("received interrupt: ");
    char s[3];
    int_to_ascii(r.int_no, s);
    kprint(s);
    kprint("\n");
    kprint(exception_messages[r.int_no]);
    kprint("\n");
}
