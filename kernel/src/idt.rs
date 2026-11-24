use crate::spinlock::Spinlock;
use core::mem::size_of;

/// The 64-bit kernel code segment selector (CODE_SEG64 in boot/long_mode.asm).
const KERNEL_CODE_SELECTOR: u16 = 0x08;

/// One IDT gate descriptor, laid out exactly as the CPU expects.
#[derive(Clone, Copy)]
#[repr(C)]
struct IdtEntry {
    offset_low: u16,  // handler address bits 0..15
    selector: u16,    // code segment to run the handler in
    ist: u8,          // interrupt stack table index (0 = don't switch stacks)
    type_attr: u8,    // present bit, privilege level, gate type
    offset_mid: u16,  // handler address bits 16..31
    offset_high: u32, // handler address bits 32..63
    reserved: u32,    // must be zero
}

const _: () = assert!(size_of::<IdtEntry>() == 16);

impl IdtEntry {
    /// An empty entry: not present, so the CPU won't use it.
    const fn missing() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// An entry that sends this vector to the code at address `handler`.
    fn new(handler: u64) -> Self {
        IdtEntry {
            offset_low: handler as u16,
            selector: KERNEL_CODE_SELECTOR,
            ist: 0,
            type_attr: 0x8E,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

/// The Interrupt Descriptor Table: one entry per vector, 0 to 255.
pub struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    /// Creates a table where every entry is missing.
    pub const fn new() -> Self {
        Idt {
            entries: [IdtEntry::missing(); 256],
        }
    }

    /// Sends interrupt `vector` to the code at address `handler`.
    pub fn set_handler(&mut self, vector: u8, handler: u64) {
        self.entries[vector as usize] = IdtEntry::new(handler);
    }
}

/// The 10-byte value `lidt` loads: table size minus one, then table address.
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

const _: () = assert!(size_of::<IdtPointer>() == 10);

/// The kernel's interrupt table.
pub static IDT: Spinlock<Idt> = Spinlock::new(Idt::new());

/// Tells the CPU to use `IDT` for interrupts and exceptions.
pub fn load() {
    let idt = IDT.lock();
    let ptr = IdtPointer {
        limit: (size_of::<Idt>() - 1) as u16,
        base: &*idt as *const Idt as u64,
    };
    // SAFETY: `IDT` is a static, so the table stays at this address for as long as
    // the kernel runs. Every entry is either not present or points to a handler.
    unsafe {
        core::arch::asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
    }
}
