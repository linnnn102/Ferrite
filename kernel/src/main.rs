#![no_std]
#![no_main]

mod port;
mod vga;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn kmain() -> ! {
    vga::clear_screen();
    vga::print_at(0, 0, b"Hello from Rust!", 0x0f);
    vga::print_at(1, 0, b"Ferrite kernel", 0x1e);
    vga::set_cursor(1, 14);
    loop {
        // SAFETY: `hlt` only pauses the CPU until the next interrupt arrives;
        // it doesn't touch memory or registers.
        unsafe { core::arch::asm!("hlt") }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
