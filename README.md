# Ferrite

A small x86-64 operating system written from scratch: a two-stage NASM bootloader that brings the CPU from 16-bit real mode up to 64-bit long mode, and a freestanding `#![no_std]` Rust kernel.

No GRUB, no `bootloader` crate, no external Rust dependencies.

## Boot sequence

```
BIOS ──► Stage 1 (0x7C00, 512 B)
           • zero segment registers, set up stack
           • int 0x13: load stage 2 → 0x7E00, kernel → 0x10000
         Stage 2 (0x7E00)
           • load GDT, set CR0.PE ──► 32-bit protected mode
           • CPUID: check long-mode support
           • build page tables (PML4 → PDPT → PD, one 2 MiB identity page)
           • CR4.PAE, CR3, EFER.LME, CR0.PG, 64-bit GDT ──► 64-bit long mode
         call 0x10000 ──► kmain() in Rust
```

## Kernel features

- **Freestanding Rust**: `no_std`, `no_main`, `panic = "abort"`, custom panic handler, and a linker script that pins `kmain` at `0x10000`.
- **Port I/O**: a `Port` type that wraps the `in`/`out` instructions with inline `asm!`.
- **VGA text driver** (80×25 at `0xB8000`):
  - 16 colors
  - line wrapping and scrolling
  - non-printable bytes shown as `■`
  - volatile, bounds-checked buffer access
  - hardware cursor control through the CRTC ports `0x3D4`/`0x3D5`

## Project layout

```
boot/
  bootsect.asm     stage 1 + stage 2 entry, mode transitions
  disk.asm         BIOS int 0x13 sector loading
  gdt.asm          32-bit GDT
  switch_pm.asm    real → protected mode
  long_mode.asm    CPUID check, page tables, protected → long mode, 64-bit GDT
  print*.asm       BIOS / VGA print helpers
kernel/
  src/main.rs      kmain, panic handler
  src/port.rs      x86 port I/O
  src/vga.rs       VGA text-mode driver
  linker.ld        flat layout starting at 0x10000
Makefile           build, run, debug
```

## Requirements

- [NASM](https://www.nasm.us/)
- Rust with the bare-metal target and LLVM tools:
  ```sh
  rustup target add x86_64-unknown-none
  rustup component add llvm-tools-preview
  cargo install cargo-binutils   # provides rust-objcopy
  ```
- QEMU (`qemu-system-x86_64`)

On macOS: `brew install nasm qemu`.

## Build & run

```sh
make          # build build/os.img
make run      # boot in QEMU (interrupt/reset log → build/qemu.log)
make debug    # boot paused, GDB stub on :1234  (lldb: gdb-remote 1234)
make clean
```

The disk image is laid out as `[boot sector][stage 2][kernel]`. The build fails if the kernel grows larger than the `KERNEL_SECTORS` the bootloader loads. To raise the limit, update `KERNEL_SECTORS` in both `Makefile` and `boot/bootsect.asm`.

## Roadmap

- [x] Two-stage bootloader into long mode
- [x] Rust kernel entry
- [x] VGA text driver with cursor, colors, scrolling
- [ ] `print!` / `println!` macros
- [ ] IDT and CPU exception handlers
- [ ] PIC, timer and keyboard interrupts
- [ ] Physical memory management and kernel heap
