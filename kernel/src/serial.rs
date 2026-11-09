use crate::port::Port;
use crate::spinlock::Spinlock;

/// The global COM1 serial port used by `serial_print!` and `serial_println!`.
pub static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort::new(0x3f8));

/// A 16550 UART serial port. QEMU's `-serial stdio` forwards it to the terminal.
pub struct SerialPort {
    data: Port,        // base + 0
    int_enable: Port,  // base + 1
    fifo_ctrl: Port,   // base + 2
    line_ctrl: Port,   // base + 3
    modem_ctrl: Port,  // base + 4
    line_status: Port, // base + 5
}

impl SerialPort {
    /// Creates a handle for the UART whose first port is `base` (0x3F8 for COM1).
    /// Private, so `SERIAL` (COM1) is the only `SerialPort` that can exist.
    const fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_enable: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_status: Port::new(base + 5),
        }
    }

    /// Programs the UART: 38400 baud, 8N1, FIFO on, interrupts off.
    pub fn init(&mut self) {
        // SAFETY: these are COM1's UART registers (0x3F8-0x3FC). The writes only set
        // its speed, data format, FIFO and control lines; they don't touch memory.
        unsafe {
            self.int_enable.write(0x00); // disable UART interrupts (no IDT yet)
            self.line_ctrl.write(0x80); // DLAB on: +0/+1 now hold the speed divisor
            self.data.write(0x03); // divisor low byte: 115200 / 3 = 38400 baud
            self.int_enable.write(0x00); // divisor high byte (DLAB on, so +1 is the divisor)
            self.line_ctrl.write(0x03); // DLAB off; 8 data bits, no parity, 1 stop bit
            self.fifo_ctrl.write(0xc7); // enable FIFO, clear it, 14-byte threshold
            self.modem_ctrl.write(0x0b); // DTR + RTS on; OUT2 on (routes IRQs to the PIC later)
        }
    }

    /// Sends one byte, waiting until the UART can accept it.
    pub fn write_byte(&mut self, byte: u8) {
        // SAFETY: reading Line Status (0x3FD) only reports UART state, and writing
        // Data (0x3F8) only sends one byte out of COM1.
        unsafe {
            while self.line_status.read() & 0x20 == 0 {
                core::hint::spin_loop();
            }
            self.data.write(byte);
        }
    }
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}

/// Prints formatted text to COM1. Used by `serial_print!`/`serial_println!`; call those instead.
#[doc(hidden)]
pub fn _serial_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL.lock().write_fmt(args).unwrap();
}

/// Prints to the serial port (COM1), like `print!`.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_serial_print(format_args!($($arg)*)));
}

/// Prints to the serial port (COM1) with a newline, like `println!`.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
