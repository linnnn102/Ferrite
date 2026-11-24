use crate::idt::{self, IDT};
use crate::{println, serial_println};

// Entry stubs for CPU exceptions 0-31. Each one makes the stack look the same
// (error code + vector), then jumps to isr_common, which saves registers and
// calls exception_handler.
core::arch::global_asm!(
    ".macro ISR_NOERR num",
    "isr\\num:",
    "    push 0",     // fake error code, so every frame has the same shape
    "    push \\num", // vector number
    "    jmp isr_common",
    ".endm",
    ".macro ISR_ERR num",
    "isr\\num:",
    "    push \\num", // the CPU already pushed an error code
    "    jmp isr_common",
    ".endm",
    ".section .text",
    "ISR_NOERR 0",
    "ISR_NOERR 1",
    "ISR_NOERR 2",
    "ISR_NOERR 3",
    "ISR_NOERR 4",
    "ISR_NOERR 5",
    "ISR_NOERR 6",
    "ISR_NOERR 7",
    "ISR_ERR   8",
    "ISR_NOERR 9",
    "ISR_ERR   10",
    "ISR_ERR   11",
    "ISR_ERR   12",
    "ISR_ERR   13",
    "ISR_ERR   14",
    "ISR_NOERR 15",
    "ISR_NOERR 16",
    "ISR_ERR   17",
    "ISR_NOERR 18",
    "ISR_NOERR 19",
    "ISR_NOERR 20",
    "ISR_ERR   21",
    "ISR_NOERR 22",
    "ISR_NOERR 23",
    "ISR_NOERR 24",
    "ISR_NOERR 25",
    "ISR_NOERR 26",
    "ISR_NOERR 27",
    "ISR_NOERR 28",
    "ISR_ERR   29",
    "ISR_ERR   30",
    "ISR_NOERR 31",
    "isr_common:",
    "    push rax",
    "    push rbx",
    "    push rcx",
    "    push rdx",
    "    push rsi",
    "    push rdi",
    "    push rbp",
    "    push r8",
    "    push r9",
    "    push r10",
    "    push r11",
    "    push r12",
    "    push r13",
    "    push r14",
    "    push r15",
    "    mov rdi, rsp", // 1st argument = pointer to the saved frame
    "    cld",          // the calling convention requires the direction flag clear
    "    call exception_handler",
    "    pop r15",
    "    pop r14",
    "    pop r13",
    "    pop r12",
    "    pop r11",
    "    pop r10",
    "    pop r9",
    "    pop r8",
    "    pop rbp",
    "    pop rdi",
    "    pop rsi",
    "    pop rdx",
    "    pop rcx",
    "    pop rbx",
    "    pop rax",
    "    add rsp, 16", // drop vector + error code
    "    iretq",
    ".section .rodata",
    ".balign 8",
    ".global isr_table",
    "isr_table:",
    "    .quad isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7",
    "    .quad isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15",
    "    .quad isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23",
    "    .quad isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31",
);

unsafe extern "C" {
    /// Addresses of the 32 exception stubs, defined in the assembly above.
    static isr_table: [u64; 32];
}

/// The registers saved by isr_common and the CPU, in stack order.
#[allow(dead_code)] // every field must exist to match the stack, even if unread
#[repr(C)]
struct InterruptFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    vector: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

const EXCEPTION_NAMES: [&str; 32] = [
    "Divide Error",
    "Debug",
    "Non-Maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
];

/// Called by isr_common for every CPU exception.
#[unsafe(no_mangle)]
extern "C" fn exception_handler(frame: &InterruptFrame) {
    let name = EXCEPTION_NAMES
        .get(frame.vector as usize)
        .unwrap_or(&"Unknown");
    if frame.vector == 3 {
        println!("EXCEPTION: {} at {:#x}", name, frame.rip);
        serial_println!("EXCEPTION: {} at {:#x}", name, frame.rip);
        return;
    }
    panic!(
        "EXCEPTION: {} (vector {}, error code {:#x}) at {:#x}",
        name, frame.vector, frame.error_code, frame.rip
    );
}

/// Points IDT entries 0-31 at the exception stubs, then loads the IDT.
pub fn init() {
    // SAFETY: isr_table is defined in the assembly above as 32 `.quad` addresses in
    // .rodata, matching the declared [u64; 32], and nothing ever writes to it.
    let table: &[u64; 32] = unsafe { &isr_table };
    {
        let mut idt = IDT.lock();
        for (vector, &handler) in table.iter().enumerate() {
            idt.set_handler(vector as u8, handler);
        }
    } // guard dropped here, so IDT is unlocked
    idt::load();
}
