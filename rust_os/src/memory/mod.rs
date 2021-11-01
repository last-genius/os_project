use crate::println;
use multiboot2::load;
use x86_64::{
    structures::paging::{Page, PageTable, RecursivePageTable, Translate},
    VirtAddr,
};

use self::frame_allocator::AreaFrameAllocator;

mod frame_allocator;

pub const P4: u64 = 0xffffffff_fffff000;

pub fn read_multiboot_data(multiboot_info_ptr: usize) {
    let boot_info = unsafe { load(multiboot_info_ptr as usize).unwrap() };

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    let kernel_start = elf_sections_tag
        .sections()
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag
        .sections()
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();

    let multiboot_start = multiboot_info_ptr as u64;
    let multiboot_end = multiboot_start + boot_info.total_size() as u64;

    let mut mapper = unsafe { init_tables() };
    let mut frame_allocator = unsafe {
        AreaFrameAllocator::init(
            kernel_start,
            kernel_end,
            multiboot_start,
            multiboot_end,
            memory_map_tag.memory_areas(),
        )
    };

    let addresses = [
        0xb8000,  // the identity-mapped vga buffer page
        0x201008, // some code page
        P4,       // P4 Table address
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    frame_allocator::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
}

/// Returns a mutable reference to the active level 4 table.
///
/// Since we are using recursive mapping we have to guarantee
/// that we have correctly mapped everything before calling this function,
/// hence the unsafety. Our bootloader in `boot.asm` has added the last
/// entry on the P4 table to itself, therefore looping on it 4 times
/// we are able to access the P4 table itself.
unsafe fn active_level_4_table() -> &'static mut PageTable {
    let virt: VirtAddr = VirtAddr::new(P4);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub unsafe fn init_tables() -> RecursivePageTable<'static> {
    let level_4_table = active_level_4_table();
    match RecursivePageTable::new(level_4_table) {
        Ok(a) => a,
        Err(b) => panic!("Recursive page was not setup correctly {}", b),
    }
}
