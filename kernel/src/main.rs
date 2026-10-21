#![no_std]
#![no_main]

mod port;
mod vga;

use core::fmt::Write;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn kmain() -> ! {
    let mut w = vga::Writer::at_cursor(vga::Color::White, vga::Color::Black);
    let (row, col) = vga::get_cursor();
    w.write_bytes(b"Ferrite kernel v0.1\n");

    ok(&mut w, "VGA text driver");
    ok(&mut w, "Hardware cursor");

    writeln!(w, "Kernel output starts at row {}, col {}", row, col).unwrap();

    loop {
        // SAFETY: `hlt` only pauses the CPU until the next interrupt arrives;
        // it doesn't touch memory or registers.
        unsafe { core::arch::asm!("hlt") }
    }
}

fn ok(w: &mut vga::Writer, msg: &str) {
    w.set_color(vga::Color::LightGreen, vga::Color::Black);
    w.write_bytes(b"[ OK ] ");
    w.set_color(vga::Color::LightGray, vga::Color::Black);
    w.write_bytes(msg.as_bytes());
    w.write_bytes(b"\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
