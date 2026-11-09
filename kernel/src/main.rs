#![no_std]
#![no_main]

mod port;
mod serial;
mod spinlock;
mod vga;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn kmain() -> ! {
    *vga::WRITER.lock() = vga::Writer::at_cursor(vga::Color::White, vga::Color::Black);
    serial::SERIAL.lock().init();
    let (row, col) = vga::get_cursor();
    println!("Ferrite kernel v0.1");
    println!("Kernel output starts at row {}, col {}", row, col);
    serial_println!("Ferrite: serial port ready");
    ok("VGA text driver");
    ok("Serial port (COM1)");
    ok("Hardware cursor");

    loop {
        // SAFETY: `hlt` only pauses the CPU until the next interrupt arrives;
        // it doesn't touch memory or registers.
        unsafe { core::arch::asm!("hlt") }
    }
}

/// Prints a boot status line: a green "[ OK ]" tag followed by `msg`,
/// on the screen and on the serial port.
fn ok(msg: &str) {
    let mut w = vga::WRITER.lock();
    w.set_color(vga::Color::LightGreen, vga::Color::Black);
    w.write_bytes(b"[ OK ] ");
    w.set_color(vga::Color::LightGray, vga::Color::Black);
    w.write_bytes(msg.as_bytes());
    w.write_bytes(b"\n");
    // A different lock from WRITER, so holding `w` here can't deadlock.
    serial_println!("[ OK ] {}", msg);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
