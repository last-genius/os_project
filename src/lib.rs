#![no_std]

mod vga_buffer;

use core::panic::PanicInfo;

/// The starting place of the kernel
///
/// The bootloader jumps over here once it properly sets up the long mode
/// and checks whether we were loaded by a multiboot-compliant bootloader
/// (all can be seen in `boot.asm` and `long_mode_init.asm`)
#[no_mangle]
pub extern "C" fn _start() -> ! {
    for i in 0..120 {
        println!("Hello world {}", i);
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
