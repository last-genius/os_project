arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-rust_os
rust_os := target/$(target)/debug/librust_os.a

linker_script := rust_os/src/linker.ld
grub_cfg := rust_os/src/grub.cfg
assembly_source_files := $(wildcard rust_os/src/*.asm)
assembly_object_files := $(patsubst rust_os/src/%.asm, \
	build/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	rm -r build

run: $(iso)
	qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

kernel:
	cargo build

# compile assembly files
build/%.o: rust_os/src/%.asm
	mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@
