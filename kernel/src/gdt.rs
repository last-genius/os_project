use crate::println;
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::{load_ds, set_cs};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{
    Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

// setup the stacks
pub const DOUBLE_FAULT_IST_INDEX: u8 = 0;
const STACK_SIZE: usize = 0x2000;
pub static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
pub static mut USTACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let s_start = VirtAddr::from_ptr(unsafe { &STACK });
            let s_end = s_start + STACK_SIZE;
            s_end
        };
        tss.privilege_stack_table[0] = {
            let s_start = VirtAddr::from_ptr(unsafe { &USTACK });
            let s_end = s_start + STACK_SIZE;
            s_end
        };
        tss
    };
}

// global GDT structure, initialized right at kernel startup
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, [SegmentSelector; 5]) = {
        let mut gdt = GlobalDescriptorTable::new();
        let data_flags =
            DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::WRITABLE;
        let code_selec = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selec = gdt.add_entry(Descriptor::UserSegment(kernel_data_flags.bits()));
        let tss_selec = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let user_data_selec = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selec = gdt.add_entry(Descriptor::user_code_segment());
        (
            gdt,
            [code_selec, data_selec, tss_selec, user_data_selec, user_code_selec],
        )
    };
}

// initialize the GDT and stacks
pub fn init_gdt() {
    GDT.0.load();
    let stack = unsafe { &STACK as *const _ };
    let ustack = unsafe { &USTACK as *const _ };
    println!(
        " - Loaded GDT: {:p} TSS: {:p} Stack {:p} User stack: {:p} CS segment: {} TSS segment: {}",
        &GDT.0 as *const _, &*TSS as *const _, stack, ustack, GDT.1[0].0, GDT.1[1].0
    );
    unsafe {
        set_cs(GDT.1[0]);
        load_ds(GDT.1[1]);
        load_tss(GDT.1[2]);
    }
}

// this I don't understand tbh, but it works
// found this on osdev discussions
#[inline(always)]
pub unsafe fn set_usermode_segs() -> (u16, u16) {
    // set ds and tss, return cs and ds
    let (mut _cs, mut _ds) = (GDT.1[4], GDT.1[3]);
    _cs.0 |= PrivilegeLevel::Ring3 as u16;
    _ds.0 |= PrivilegeLevel::Ring3 as u16;
    load_ds(_ds);
    (_cs.0, _ds.0)
}