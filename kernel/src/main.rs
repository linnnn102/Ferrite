#![no_std]
#![no_main]

mod idt;
mod port;
mod serial;
mod spinlock;
mod vga;

use core::panic::PanicInfo;

// The kernel's first instruction. Zeroes .bss, then jumps to kmain.
core::arch::global_asm!(
    ".section .text.entry, \"ax\"",
    ".global _start",
    "_start:",
    "    cld",
    "    lea rdi, [rip + __bss_start]",
    "    lea rcx, [rip + __bss_end]",
    "    sub rcx, rdi",
    "    xor eax, eax",
    "    rep stosb",
    "    jmp kmain",
);

#[unsafe(no_mangle)]
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
    idt::load();
    ok("IDT loaded");

    hlt_loop();
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
fn panic(info: &PanicInfo) -> ! {
    // SAFETY: single CPU with interrupts off, so any holder of these locks is the
    // code that panicked, and it never resumes to use the data again.
    unsafe {
        vga::WRITER.force_unlock();
        serial::SERIAL.force_unlock();
    }

    vga::WRITER
        .lock()
        .set_color(vga::Color::LightRed, vga::Color::Black);
    println!("KERNEL PANIC: {}", info.message());
    if let Some(loc) = info.location() {
        println!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
    }

    serial_println!("KERNEL PANIC: {}", info.message());
    if let Some(loc) = info.location() {
        serial_println!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
    }

    hlt_loop();
}

/// Halts the CPU forever.
fn hlt_loop() -> ! {
    loop {
        // SAFETY: `hlt` only pauses the CPU until the next interrupt arrives;
        // it doesn't touch memory or registers.
        unsafe { core::arch::asm!("hlt") }
    }
}
