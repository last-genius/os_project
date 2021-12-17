#![feature(abi_x86_interrupt)]
#![no_std]

mod memory;
mod vga_buffer;
mod gdt;
mod interrupts;

use core::panic::PanicInfo;

/// The starting place of the kernel
///
/// The bootloader jumps over here once it properly sets up the long mode
/// and checks whether we were loaded by a multiboot-compliant bootloader
/// (all can be seen in `boot.asm` and `long_mode_init.asm`)
#[no_mangle]
pub extern "C" fn _start(multiboot_info_ptr: usize) -> ! {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let boot_info = unsafe { multiboot2::load(multiboot_info_ptr as usize).unwrap() };
    let mut frame_allocator = memory::read_multiboot_data(multiboot_info_ptr, &boot_info);
    let mut mapper = unsafe { memory::init_tables() };
    memory::test_frame_allocation(&mut mapper, &mut frame_allocator);
    memory::test_address_translation(&mapper);

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
