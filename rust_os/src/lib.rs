#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

pub mod memory;
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;

extern crate alloc;
use alloc::vec;

use core::panic::PanicInfo;

/// The starting place of the kernel
///
/// The bootloader jumps over here once it properly sets up the long mode
/// and checks whether we were loaded by a multiboot-compliant bootloader
/// (all can be seen in `boot.asm` and `long_mode_init.asm`)

use memory::buddy_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

#[no_mangle]
pub extern "C" fn _start(multiboot_info_ptr: usize) -> ! {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let boot_info = unsafe { multiboot2::load(multiboot_info_ptr as usize).unwrap() };
    // let mut frame_allocator = memory::read_multiboot_data(multiboot_info_ptr, &boot_info);
    // print!("{:#?}", frame_allocator);
    // let mut mapper = unsafe { memory::init_tables() };
    // memory::test_frame_allocation(&mut mapper, &mut frame_allocator);
    // memory::test_address_translation(&mapper);

    let multiboot_start = multiboot_info_ptr as usize;
    let multiboot_end = multiboot_start + boot_info.total_size() as usize;
    unsafe {
        HEAP_ALLOCATOR.lock().init(multiboot_end, 131_072);
    }
    println!("Allocator set!");

    let test = vec![1,2,3];

    for x in &test {
        println!("{}", x);
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
