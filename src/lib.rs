#![no_std]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"hello world ";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for i in 0..120 {
        for (j, &byte) in HELLO.iter().enumerate() {
            unsafe {
                *vga_buffer.offset(2 * i * HELLO.len() as isize + j as isize * 2) = byte;
                *vga_buffer.offset(2 * i * HELLO.len() as isize + j as isize * 2 + 1) = 0xb;
            }
        }
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
