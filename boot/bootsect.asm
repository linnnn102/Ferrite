[org 0x7c00] ; bootloader offset
STAGE2_SECTORS equ 2       ; size of stage 2 (everything after the boot sector)
KERNEL_OFFSET  equ 0x10000 ; where the kernel is loaded (must match linker.ld)
KERNEL_SECTORS equ 32      ; how many 512-byte sectors to load (must match Makefile)

; ===== Stage 1: the 512-byte boot sector the BIOS loads at 0x7c00 =====
[bits 16]
    xor ax, ax          ; segment registers may hold garbage at boot: zero them
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov [BOOT_DRIVE], dl ; BIOS stores the boot drive number in dl

    mov bp, 0x9000 ; set the stack
    mov sp, bp

    mov bx, MSG_REAL_MODE
    call print
    call print_nl

    ; load stage 2 right after the boot sector, so it lands at 0x7e00
    mov bx, 0x7e00      ; ES:BX = 0x0000:0x7e00
    mov cl, 2           ; disk sector 2 (sector 1 is this boot sector)
    mov dh, STAGE2_SECTORS
    mov dl, [BOOT_DRIVE]
    call disk_load

    ; load the kernel at 0x10000 while the BIOS is still available
    mov ax, KERNEL_OFFSET >> 4 ; ES:BX = 0x1000:0x0000 = 0x10000
    mov es, ax
    xor bx, bx
    mov cl, 2 + STAGE2_SECTORS ; first sector after stage 2
    mov dh, KERNEL_SECTORS
    mov dl, [BOOT_DRIVE]
    call disk_load

    call switch_to_pm ; lives in stage 2, which is now in memory
    jmp $ ; this will actually never be executed

%include "print.asm"
%include "print_hex.asm"
%include "disk.asm"

BOOT_DRIVE    db 0
MSG_REAL_MODE db "16-bit real mode", 0

times 510-($-$$) db 0 ; pad the boot sector to 510 bytes
dw 0xaa55             ; boot signature

; ===== Stage 2: loaded at 0x7e00 by stage 1 =====
%include "gdt.asm"
%include "32bit-print.asm"
%include "switch_pm.asm"
%include "long_mode.asm"

[bits 32]
BEGIN_PM: ; after the switch we will get here
    mov ebx, MSG_PROT_MODE
    call print_string_pm ; Note that this will be written at the top left corner
    call check_long_mode
    call setup_page_tables
    jmp enable_long_mode

[bits 64]
BEGIN_LM:
    mov ax, DATA_SEG64
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, 0x90000

    call KERNEL_OFFSET ; jump into Rust: kmain()
    jmp $              ; kmain never returns, but just in case

MSG_PROT_MODE db "32-bit protected mode", 0
MSG_NO_LM     db "No long mode", 0

times (1 + STAGE2_SECTORS) * 512 - ($-$$) db 0 ; pad stage 2 to whole sectors
