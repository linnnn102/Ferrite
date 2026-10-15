#![no_std]
#![no_main]

mod port;
mod vga;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn kmain() -> ! {
    vga::clear_screen();
    let mut w = vga::Writer::new(vga::Color::LightGreen, vga::Color::Black);

    // 30 lines on a 25-line screen: the first 5 + 3 should scroll off
    for i in 0..30u8 {
        w.write_byte(b'A' + i % 26);
        w.write_bytes(b" line\n");
    }

    // 100 characters on an 80-column screen: should wrap onto a second row
    w.set_color(vga::Color::Yellow, vga::Color::Blue);
    w.write_bytes(&[b'x'; 100]);

    // \xe9 isn't printable ASCII: should appear as ■
    w.write_bytes(b"\n\xe9 end");
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
