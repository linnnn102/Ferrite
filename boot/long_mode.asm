[bits 32]
; 1. Does the CPU support long mode?
check_long_mode:
    mov eax, 0x80000000     ; ask for the highest extended CPUID function
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29       ; LM bit
    jz .no_long_mode
    ret
.no_long_mode:
    mov ebx, MSG_NO_LM
    call print_string_pm
    jmp $

; 2. Page tables: PML4 @ 0x1000 -> PDPT @ 0x2000 -> PD @ 0x3000
;    PD[0] = one 2 MiB huge page identity-mapping 0x0 - 0x1FFFFF
setup_page_tables:
    mov edi, 0x1000
    xor eax, eax
    mov ecx, 3 * 4096 / 4   ; zero 3 tables
    rep stosd
    mov dword [0x1000], 0x2003  ; PML4[0] -> PDPT, present + writable
    mov dword [0x2000], 0x3003  ; PDPT[0] -> PD,   present + writable
    mov dword [0x3000], 0x83    ; PD[0]: present + writable + huge (bit 7)
    ret

; 3 + 4. PAE, CR3, EFER.LME, paging, then far jump into 64-bit code
enable_long_mode:
    mov eax, cr4
    or eax, 1 << 5          ; CR4.PAE
    mov cr4, eax

    mov eax, 0x1000
    mov cr3, eax            ; CR3 = address of PML4

    mov ecx, 0xC0000080     ; EFER MSR
    rdmsr
    or eax, 1 << 8          ; EFER.LME
    wrmsr

    mov eax, cr0
    or eax, 1 << 31         ; CR0.PG -> long mode becomes active
    mov cr0, eax

    lgdt [gdt64_descriptor]
    jmp CODE_SEG64:BEGIN_LM

; 64-bit GDT: same as before, except the code segment sets L=1, D=0
gdt64_start:
    dq 0
gdt64_code:
    dw 0xffff, 0
    db 0, 10011010b, 10101111b, 0   ; flags nibble 1010 = G=1, D=0, L=1
gdt64_data:
    dw 0xffff, 0
    db 0, 10010010b, 11001111b, 0
gdt64_end:
gdt64_descriptor:
    dw gdt64_end - gdt64_start - 1
    dd gdt64_start
CODE_SEG64 equ gdt64_code - gdt64_start
DATA_SEG64 equ gdt64_data - gdt64_start
