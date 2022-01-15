use crate::gdt;
use crate::mem;
use crate::serial_println;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Display;
use lazy_static::lazy_static;
use spin::Mutex;

// saved register values under context change
#[derive(Debug, Clone)]
pub struct Context {
    pub rbp: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

// fill the Context struct by pushing register values onto a stack
// the stack pointer is assigned to the struct
#[inline(always)]
pub unsafe fn get_context() -> *const Context {
    let ctx_ptr: *const Context;
    asm!("push r15; push r14; push r13; push r12; push r11; push r10; push r9;\
    push r8; push rdi; push rsi; push rdx; push rcx; push rbx; push rax; push rbp;\
    mov {}, rsp; sub rsp, 0x400;",
    out(reg) ctx_ptr);
    ctx_ptr
}

// reverse get_context
#[inline(always)]
pub unsafe fn restore_context(ctx_ref: &Context) {
    asm!("mov rsp, {};\
    pop rbp; pop rax; pop rbx; pop rcx; pop rdx; pop rsi; pop rdi; pop r8; pop r9;\
    pop r10; pop r11; pop r12; pop r13; pop r14; pop r15; iretq;",
    in(reg) ctx_ref);
}

#[inline(never)]
pub unsafe fn jmp_to_usermode(code: mem::VirtAddr, stack_end: mem::VirtAddr) {
    let (cs, ds) = gdt::set_usermode_segs();
    x86_64::instructions::tlb::flush_all();
    asm!("\
    push rax   // stack segment
    push rsi   // rsp
    push 0x200 // rflags (only interrupt bit set)
    push rdx   // code segment
    push rdi   // ret to virtual addr
    iretq",
    in("rdi") code.addr(), in("rsi") stack_end.addr(), in("dx") cs, in("ax") ds);
}

// tasks (processes) can have either a saved context, or a stack/instruction pointer
#[derive(Clone, Debug)]
enum TaskState {
    SavedContext(Context),
    StartingInfo(mem::VirtAddr, mem::VirtAddr),
}

struct Task {
    state: TaskState,
    ptable: Box<mem::PageTable>,
    stack_space: Vec<u8>, // container for stack space
}

impl Task {
    pub fn new(
        base: mem::VirtAddr,
        stack_top: mem::VirtAddr,
        stack_space: Vec<u8>,
        ptable: Box<mem::PageTable>,
    ) -> Task {
        Task {
            state: TaskState::StartingInfo(base, stack_top),
            stack_space,
            ptable,
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe {
            write!(
                f,
                "PT: {}, Context: {:x?}",
                self.ptable.phys_addr(),
                self.state
            )
        }
    }
}

pub struct Scheduler {
    tasks: Mutex<Vec<Task>>,
    cur_task: Mutex<Option<usize>>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            tasks: Mutex::new(Vec::new()),
            cur_task: Mutex::new(None), // so that next task is 0
        }
    }

    // schedule a task
    pub unsafe fn schedule(&self, fn_addr: mem::VirtAddr) {
        let fn_phys = fn_addr.to_phys().unwrap().0; // convert to physical address
        let page_start = (fn_phys.addr() >> 12) << 12; // zero out page offset to get which page we should map
        let fn_page_offset = fn_phys.addr() - page_start; // offset of function from page start
        let fn_virt_base = 0x400000; // target virtual address of page
        let fn_virt = fn_virt_base + fn_page_offset; // target virtual address of function
        serial_println!(
            "Mapping {:x} to {:x}",
            page_start,
            fn_virt_base
        );
        let mut ptable = mem::PageTable::new(); // copy over the kernel's page tables
        ptable.map_virt_to_phys(
            mem::VirtAddr::new(fn_virt_base),
            mem::PhysAddr::new(page_start),
            mem::BIT_PRESENT | mem::BIT_USER,
        ); // map the program's code
        ptable.map_virt_to_phys(
            mem::VirtAddr::new(fn_virt_base).offset(0x1000),
            mem::PhysAddr::new(page_start).offset(0x1000),
            mem::BIT_PRESENT | mem::BIT_USER,
        ); // also map another page to be sure we got the entire function in
        let mut stack_space: Vec<u8> = Vec::with_capacity(0x1000); // allocate some memory to use for the stack
        let stack_space_phys = mem::VirtAddr::new(stack_space.as_mut_ptr() as *const u8 as u64)
            .to_phys()
            .unwrap()
            .0;
        // take physical address of stack
        ptable.map_virt_to_phys(
            mem::VirtAddr::new(0x800000),
            stack_space_phys,
            mem::BIT_PRESENT | mem::BIT_WRITABLE | mem::BIT_USER,
        ); // map the stack memory to 0x800000
        let task = Task::new(
            mem::VirtAddr::new(fn_virt),
            mem::VirtAddr::new(0x801000),
            stack_space,
            ptable,
        ); // create task struct
        self.tasks.lock().push(task); // push task struct to list of tasks
    }

    // replace the context of the current task if one exists
    pub unsafe fn save_current_context(&self, ctx_ptr: *const Context) {
        self.cur_task.lock().map(|cur_task_idx| {
            let ctx = (*ctx_ptr).clone();
            self.tasks.lock()[cur_task_idx].state = TaskState::SavedContext(ctx);
        });
    }

    // run the next scheduled task, either start it up or restore it if already active
    pub unsafe fn run_next(&self) {
        let tasks_len = self.tasks.lock().len();
        if tasks_len > 0 {
            let task_state = {
                let mut cur_task_o = self.cur_task.lock(); // lock the current task
                let cur_task = cur_task_o.get_or_insert(0);
                let next_task = (*cur_task + 1) % tasks_len;
                *cur_task = next_task;
                let task = &self.tasks.lock()[next_task]; // get the next task
                serial_println!("Switching to task #.{} ({})", next_task, task);
                task.ptable.enable();
                task.state.clone()
            };
            // continue based on task state
            match task_state {
                TaskState::SavedContext(ctx) => {
                    restore_context(&ctx)
                }
                TaskState::StartingInfo(base, stack_top) => {
                    jmp_to_usermode(base, stack_top)
                }
            }
        }
    }
}

lazy_static! {
    pub static ref SCHEDULER: Scheduler = Scheduler::new();
}