use multiboot2::MemoryArea;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, RecursivePageTable, Size4KiB},
    PhysAddr,
};

const PAGE_SIZE: u64 = 4096;

pub struct AreaFrameAllocator<'a, T>
where
    T: Iterator<Item = &'a MemoryArea> + Clone,
{
    next_free_frame: PhysFrame,
    current_area: Option<&'a MemoryArea>,
    areas: T,
    kernel_start: PhysFrame,
    kernel_end: PhysFrame,
    multiboot_start: PhysFrame,
    multiboot_end: PhysFrame,
}

impl<'a, T> AreaFrameAllocator<'a, T>
where
    T: Iterator<Item = &'a MemoryArea> + Clone,
{
    pub unsafe fn init(
        kernel_start: u64,
        kernel_end: u64,
        multiboot_start: u64,
        multiboot_end: u64,
        memory_areas: T,
    ) -> AreaFrameAllocator<'a, T> {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: PhysFrame::containing_address(PhysAddr::new(kernel_start)),
            kernel_end: PhysFrame::containing_address(PhysAddr::new(kernel_end)),
            multiboot_start: PhysFrame::containing_address(PhysAddr::new(multiboot_start)),
            multiboot_end: PhysFrame::containing_address(PhysAddr::new(multiboot_end)),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self
            .areas
            .clone()
            .filter(|area| {
                let address = area.start_address() + area.size() - 1;
                PhysFrame::containing_address(PhysAddr::new(address)) >= self.next_free_frame
            })
            .min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = PhysFrame::containing_address(PhysAddr::new(area.start_address()));
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

unsafe impl<'a, T> FrameAllocator<Size4KiB> for AreaFrameAllocator<'a, T>
where
    T: Iterator<Item = &'a MemoryArea> + Clone,
{
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free.
            let frame = self.next_free_frame.clone();

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.start_address() + area.size() - 1;
                PhysFrame::containing_address(PhysAddr::new(address))
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame = PhysFrame::containing_address(PhysAddr::new(
                    self.kernel_start.start_address().as_u64() + PAGE_SIZE,
                ));
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = PhysFrame::containing_address(PhysAddr::new(
                    self.multiboot_end.start_address().as_u64() + PAGE_SIZE,
                ));
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame = PhysFrame::containing_address(PhysAddr::new(
                    self.next_free_frame.start_address().as_u64() + PAGE_SIZE,
                ));
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut RecursivePageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
