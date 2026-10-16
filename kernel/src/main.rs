#![no_std]
#![no_main]

mod port;
mod vga;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn kmain() -> ! {
    let mut w = vga::Writer::at_cursor(vga::Color::White, vga::Color::Black);
    w.write_bytes(b"Ferrite kernel v0.1\n");

    w.set_color(vga::Color::LightGreen, vga::Color::Black);
    w.write_bytes(b"[ OK ] ");
    w.set_color(vga::Color::LightGray, vga::Color::Black);
    w.write_bytes(b"VGA text driver\n");

    w.set_color(vga::Color::LightGreen, vga::Color::Black);
    w.write_bytes(b"[ OK ] ");
    w.set_color(vga::Color::LightGray, vga::Color::Black);
    w.write_bytes(b"Hardware cursor\n");

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
