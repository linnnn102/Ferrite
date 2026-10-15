use core::arch::asm;

/// Reads one byte from an I/O port.
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: `in` only touches the al/dx registers named below; the caller
    // is responsible for what reading this port does to the device.
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port,
             options(nomem, nostack, preserves_flags));
    }
    value
}

/// Writes one byte to an I/O port.
unsafe fn outb(port: u16, value: u8) {
    // SAFETY: `out` only touches the al/dx registers named below; the caller
    // is responsible for what writing this port does to the device.
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value,
             options(nomem, nostack, preserves_flags));
    }
}

/// A single 8-bit I/O port.
pub struct Port {
    port: u16,
}

impl Port {
    /// Creates a handle for port number `port`.
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    /// Reads a byte from this port.
    ///
    /// # Safety
    /// Reading a port can change hardware state (for example, reading the
    /// keyboard port removes the key from its buffer). The caller must make
    /// sure reading this port is valid for the device behind it.
    pub unsafe fn read(&self) -> u8 {
        // SAFETY: the caller upholds this function's safety contract.
        unsafe { inb(self.port) }
    }

    /// Writes a byte to this port.
    ///
    /// # Safety
    /// Writing a port can reconfigure hardware, reset the machine or corrupt
    /// a disk. The caller must make sure `value` is valid for this port's device.
    pub unsafe fn write(&self, value: u8) {
        // SAFETY: the caller upholds this function's safety contract.
        unsafe { outb(self.port, value) }
    }
}
