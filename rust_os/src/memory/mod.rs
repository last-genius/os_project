#![allow(dead_code)]

use crate::println;
use multiboot2::BootInformation;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTable, PhysFrame, RecursivePageTable, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use self::frame_allocator::{AreaFrameAllocator, MemoryArea};
use self::buddy_allocator::Heap;

mod frame_allocator;
pub mod buddy_allocator;
mod linked_list;

pub const P4: u64 = 0xffffffff_fffff000;
pub const ORD: usize = 64;

pub fn read_multiboot_data(
    multiboot_info_ptr: usize,
    boot_info: &BootInformation,
) -> Heap<ORD> {
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

    let multiboot_start = multiboot_info_ptr as usize; // CHANGE TO u64 FOR FRAME_ALLOCATOR!!!!!!
    let multiboot_end = multiboot_start + boot_info.total_size() as usize;

    // let frame_allocator = unsafe { /// Original, working but plain version
    //     AreaFrameAllocator::init(
    //         kernel_start,
    //         kernel_end,
    //         multiboot_start,
    //         multiboot_end,
    //         memory_map_tag.memory_areas(),
    //     )
    // };

    let mut buddy_allocator = Heap::empty();
    unsafe {
        buddy_allocator.init(multiboot_end, 4294967296);
    }
    buddy_allocator
}

pub fn test_address_translation(mapper: &RecursivePageTable) {
    let addresses = [
        0x201008,      // some code page
        P4,            // P4 Table address
        0xdeadbeaf000, // newly mapped VGA buffer
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
}

pub fn test_frame_allocation(
    mapper: &mut RecursivePageTable,
    frame_allocator: &mut Heap<ORD>,
)
{
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    create_example_mapping(page, mapper);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(100).write_volatile(0x_f021_f077_f065_f04e) };
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut RecursivePageTable,
    // frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    // use x86_64::structures::paging::PageTableFlags as Flags;
    //
    // let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    // let flags = Flags::PRESENT | Flags::WRITABLE;
    //
    // let map_to_result = unsafe {
    //     // FIXME: this is not safe, we do it only for testing
    //     mapper.map_to(page, frame, flags, frame_allocator)
    // };
    // map_to_result.expect("map_to failed").flush();
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
