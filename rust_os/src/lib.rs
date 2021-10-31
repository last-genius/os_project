#![no_std]

mod memory;
mod vga_buffer;

use core::panic::PanicInfo;

/// The starting place of the kernel
///
/// The bootloader jumps over here once it properly sets up the long mode
/// and checks whether we were loaded by a multiboot-compliant bootloader
/// (all can be seen in `boot.asm` and `long_mode_init.asm`)
#[no_mangle]
pub extern "C" fn _start(multiboot_info_ptr: usize) -> ! {
    memory::read_multiboot_data(multiboot_info_ptr);
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
