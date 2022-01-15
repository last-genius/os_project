// TODO: document further

use crate::port::{end_of_interrupt, Port};
use crate::scheduler;
use crate::{print, println};
use lazy_static::lazy_static;
use spin::Mutex;

use x86_64::instructions::segmentation;
use x86_64::instructions::tables::{lidt, DescriptorTablePointer};
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::InterruptStackFrame;

use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1};

use core::mem::size_of;

type IDTHandler = extern "x86-interrupt" fn();

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
}

macro_rules! irq_fn {
    ($f: ident, $i: literal, $e:expr) => {
        unsafe extern "x86-interrupt" fn $f(_sframe: &mut InterruptStackFrame) {
            asm!("cli");
            $e();
            end_of_interrupt($i);
            asm!("sti");
        }
    }
}

extern "x86-interrupt" fn div_by_zero(sframe: &mut InterruptStackFrame) {
    println!("division by zero! {:?}", sframe);
}

extern "x86-interrupt" fn breakpoint(sframe: &mut InterruptStackFrame) {
    println!("int3 {:?}", sframe);
}

extern "x86-interrupt" fn page_fault(sframe: &mut InterruptStackFrame, errno: u64) {
    println!("page fault! error code: {} {:?}", errno, sframe);
    loop {}
}

extern "x86-interrupt" fn gpf(sframe: &mut InterruptStackFrame, errno: u64) {
    println!("GPF! error code: {} {:?}", errno, sframe);
    loop {}
}

extern "x86-interrupt" fn double_fault(sframe: &mut InterruptStackFrame, errno: u64) {
    println!("double fault! error code: {} {:?}", errno, sframe);
    loop {}
}

#[naked]
unsafe extern "C" fn timer(_sframe: &mut InterruptStackFrame) {
    let ctx = scheduler::get_context();
    scheduler::SCHEDULER.save_current_context(ctx);
    end_of_interrupt(32);
    scheduler::SCHEDULER.run_next();
}

// handler for detecting and returning keystrokes
irq_fn!(keyboard, 33, || {
    let port: Port<u8> = Port::new(0x60);
    let scan_code = port.read();
    let mut kbd = KEYBOARD.lock();
    if let Ok(Some(key_event)) = kbd.add_byte(scan_code) {
        if let Some(key) = kbd.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(char) => print!("{}", char),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
});

// setup the interrupt table with 4 interrupts
lazy_static! {
    static ref INTERRUPT_TABLE: InterruptDescriptorTable = {
        let mut vectors = [IDTEntry::empty(); 0x100];
        macro_rules! idt_entry {
            ($i:literal, $e:expr) => {
                vectors[$i] =
                    IDTEntry::new($e as *const IDTHandler, segmentation::cs(), 0, true, 0);
            };
        }
        idt_entry!(0, div_by_zero);
        idt_entry!(3, breakpoint);
        vectors[8] = IDTEntry::new(
            double_fault as *const IDTHandler,
            segmentation::cs(),
            crate::gdt::DOUBLE_FAULT_IST_INDEX + 1,
            true,
            0,
        );
        idt_entry!(13, gpf);
        idt_entry!(14, page_fault);
        idt_entry!(32, timer);
        idt_entry!(33, keyboard);
        InterruptDescriptorTable(vectors)
    };
}

#[repr(C, packed)]
struct InterruptDescriptorTable([IDTEntry; 0x100]);

impl InterruptDescriptorTable {
    fn load(&'static self) {
        let idt_ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };
        println!(" - Setting up IDT with {} entries", INTERRUPT_TABLE.0.len());
        println!(" - IDT ptr address: {:x}", &idt_ptr as *const _ as u64);
        println!(
            " - IDT address: {:x}",
            &INTERRUPT_TABLE.0 as *const _ as u64
        );
        unsafe {
            lidt(&idt_ptr);
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct IDTEntry {
    handler_bottom: u16,
    gdt_selector: u16,
    interrupt_options: u16,
    handler_mid: u16,
    handler_top: u32,
    reserved: u32,
}

impl IDTEntry {
    fn new(
        handler: *const IDTHandler,
        gdt_selector: SegmentSelector,
        int_stack_idx: u8,
        disable_interrupts: bool,
        dpl_private: u8,
    ) -> IDTEntry {
        let mut interrupt_options: u16 = int_stack_idx as u16 & 0b111;
        if !disable_interrupts {
            interrupt_options |= 1 << 8;
        }
        interrupt_options |= 1 << 9;
        interrupt_options |= 1 << 10;
        interrupt_options |= 1 << 11;
        interrupt_options |= (dpl_private as u16 & 0b11) << 13;
        interrupt_options |= 1 << 15;
        let hdlr_ptr = handler as u64;
        let handler_bottom = (hdlr_ptr & 0xFFFF) as u16;
        let handler_mid = ((hdlr_ptr >> 16) & 0xFFFF) as u16;
        let handler_top = (hdlr_ptr >> 32) as u32;
        let gdt_selector = gdt_selector.0;
        IDTEntry {
            handler_bottom,
            handler_mid,
            handler_top,
            interrupt_options,
            gdt_selector,
            reserved: 0,
        }
    }

    fn empty() -> IDTEntry {
        IDTEntry {
            handler_bottom: 0,
            handler_mid: 0,
            handler_top: 0,
            interrupt_options: 0,
            gdt_selector: segmentation::cs().0,
            reserved: 0,
        }
    }
}

pub fn setup_idt() {
    INTERRUPT_TABLE.load();
}