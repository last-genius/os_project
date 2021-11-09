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

C_SOURCES = $(wildcard rust_os/src/kernel/*.c rust_os/src/drivers/*.c rust_os/src/cpu/*.c libc/*.c)
HEADERS = $(wildcard rust_os/src/kernel/*.h rust_os/src/drivers/*.h rust_os/src/cpu/*.h libc/*.h)

OBJ = $(patsubst %.c,%.o,$(C_SOURCES))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	rm -r build target my_build

run: $(iso)
	qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	rm -r build/isofiles

$(kernel): kernel $(assembly_object_files)
	@ld -n -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

kernel: $(OBJ) my_build/interrupt.o
	mkdir -p target/$(target)/debug
	ar rcs $(rust_os) $^ my_build/interrupt.o 
	
my_build/%.o: rust_os/src/*/%.c $(HEADERS)
	mkdir -p my_build
	gcc -c -o $@ $<
	
my_build/interrupt.o: rust_os/src/cpu/interrupt.asm
	mkdir -p my_build
	nasm -felf64 rust_os/src/cpu/interrupt.asm -o $@

# compile assembly files
build/%.o: rust_os/src/%.asm
	mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@
