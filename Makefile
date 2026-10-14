# Number of 512-byte sectors the bootloader loads for the kernel.
# Must match KERNEL_SECTORS in boot/bootsect.asm.
KERNEL_SECTORS := 32

BUILD      := build
KERNEL_ELF := kernel/target/x86_64-unknown-none/release/kernel

.PHONY: all run debug clean kernel

all: $(BUILD)/os.img

# 1. Assemble the boot sector (-I tells nasm where %include files live)
$(BUILD)/boot.bin: boot/*.asm | $(BUILD)
	nasm -f bin -I boot/ boot/bootsect.asm -o $@

# 2. Compile the Rust kernel. Must run *inside* kernel/ so Cargo finds
#    kernel/.cargo/config.toml and the linker finds linker.ld.
#    Phony, so Cargo is always asked; Cargo itself skips work if nothing changed.
kernel:
	cd kernel && cargo build --release

# 3. Strip the ELF headers: keep only the raw bytes that get loaded at 0x10000
$(BUILD)/kernel.bin: kernel | $(BUILD)
	rust-objcopy -O binary $(KERNEL_ELF) $@
	@size=$$(wc -c < $@); max=$$(( $(KERNEL_SECTORS) * 512 )); \
	if [ $$size -gt $$max ]; then \
		echo "error: kernel.bin is $$size bytes, but only $$max are loaded. Raise KERNEL_SECTORS."; \
		exit 1; \
	fi

# 4. Disk image: [boot sector][stage 2][kernel ... zero padding]
#    boot.bin already holds stage 1 + stage 2, padded to whole sectors,
#    so the kernel starts at sector (size of boot.bin / 512).
$(BUILD)/os.img: $(BUILD)/boot.bin $(BUILD)/kernel.bin
	@boot=$$(( $$(wc -c < $(BUILD)/boot.bin) / 512 )); \
	dd if=/dev/zero          of=$@ bs=512 count=$$(( boot + $(KERNEL_SECTORS) )) 2>/dev/null; \
	dd if=$(BUILD)/boot.bin   of=$@ conv=notrunc 2>/dev/null; \
	dd if=$(BUILD)/kernel.bin of=$@ bs=512 seek=$$boot conv=notrunc 2>/dev/null
	@echo "built $@"

run: $(BUILD)/os.img
	qemu-system-x86_64 -drive format=raw,file=$< -no-reboot -d int,cpu_reset -D $(BUILD)/qemu.log

# Starts paused and waits for a debugger on port 1234 (lldb: gdb-remote 1234)
debug: $(BUILD)/os.img
	qemu-system-x86_64 -drive format=raw,file=$< -no-reboot -s -S

$(BUILD):
	mkdir -p $@

clean:
	rm -rf $(BUILD)
	cd kernel && cargo clean
