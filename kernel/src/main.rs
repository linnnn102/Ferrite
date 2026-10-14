#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn kmain() -> ! {
    let vga = 0xb8000 as *mut u8;
    for (i, &b) in b"Hello from Rust!".iter().enumerate() {
        unsafe {
            vga.add(i * 2).write_volatile(b);        // character
            vga.add(i * 2 + 1).write_volatile(0x0f); // white on black
        }
    }
    loop { unsafe { core::arch::asm!("hlt") } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}