#include "../cpu/isr.h"
#include "../drivers/keyboard.h"

void main() {
    isr_install();

    asm volatile("sti");
    /* Comment out the timer IRQ handler to read
     * the keyboard IRQs easier */
    init_keyboard();
}
