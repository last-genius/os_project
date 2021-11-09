A patched version of the [multiboot2](https://github.com/rust-osdev/multiboot2/) crate

The original `multiboot2::memory_map::MemoryMapTag::memory_areas`
method returns an `impl Iterator<Item = &MemoryArea>`,
and I needed it to return `impl Iterator<Item = &MemoryArea> + Clone` 
so we can clone the iterator in our modules.

This is basically it, the only patched code is in `./src/memory_map.rs`.
