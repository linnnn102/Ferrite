use crate::port::Port;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

const CRTC_INDEX: Port = Port::new(0x3d4);
const CRTC_DATA: Port = Port::new(0x3d5);
const BLANK_ATTR: u8 = color_code(Color::White, Color::Black);

/// Fills the whole screen with spaces, white on black.
#[allow(dead_code)] // will be used by the shell's `clear` command later 
pub fn clear_screen() {
    for cell in 0..WIDTH * HEIGHT {
        write_cell(cell, b' ', BLANK_ATTR);
    }
}

/// Writes one character and its color to cell number `cell` (0 = top-left).
/// Does nothing if `cell` is outside the screen.
fn write_cell(cell: usize, ch: u8, attr: u8) {
    if cell >= WIDTH * HEIGHT {
        return;
    }
    // SAFETY: cell < WIDTH * HEIGHT (checked above), so both writes stay inside
    // the 4000-byte VGA text buffer at 0xb8000, which the bootloader identity-maps.
    unsafe {
        VGA_BUFFER.add(cell * 2).write_volatile(ch);
        VGA_BUFFER.add(cell * 2 + 1).write_volatile(attr);
    }
}

/// Reads the character and color stored in cell number `cell`.
/// Returns a blank white-on-black cell if `cell` is outside the screen.
fn read_cell(cell: usize) -> (u8, u8) {
    if cell >= WIDTH * HEIGHT {
        return (b' ', 0x0f);
    }
    // SAFETY: cell < WIDTH * HEIGHT (checked above), so both reads stay inside
    // the 4000-byte VGA text buffer at 0xb8000, which the bootloader identity-maps.
    unsafe {
        (
            VGA_BUFFER.add(cell * 2).read_volatile(),
            VGA_BUFFER.add(cell * 2 + 1).read_volatile(),
        )
    }
}

/// Moves the blinking hardware cursor to row `y`, column `x`.
/// Does nothing if the position is outside the screen.
pub fn set_cursor(y: usize, x: usize) {
    if y >= HEIGHT || x >= WIDTH {
        return;
    }
    let cell = y * WIDTH + x;
    // SAFETY: 0x3D4/0x3D5 are the VGA CRTC ports; registers 0x0E/0x0F only set the
    // cursor's screen position, so no value written here can affect memory.
    unsafe {
        CRTC_INDEX.write(0x0f);
        CRTC_DATA.write((cell & 0xff) as u8);
        CRTC_INDEX.write(0x0e);
        CRTC_DATA.write(((cell >> 8) & 0xff) as u8);
    }
}

/// Returns the hardware cursor's position as `(row, column)`.
pub fn get_cursor() -> (usize, usize) {
    // SAFETY: 0x3D4/0x3D5 are the VGA CRTC ports; selecting registers 0x0E/0x0F
    // and reading them back only reports the cursor position.
    let cell = unsafe {
        CRTC_INDEX.write(0x0f);
        let low = CRTC_DATA.read() as u16;
        CRTC_INDEX.write(0x0e);
        let high = CRTC_DATA.read() as u16;
        (high << 8) | low
    } as usize;
    (cell / WIDTH, cell % WIDTH)
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Combines a foreground and background color into a VGA attribute byte.
const fn color_code(fg: Color, bg: Color) -> u8 {
    ((bg as u8) << 4) | (fg as u8)
}

pub struct Writer {
    row: usize,
    col: usize,
    attr: u8,
}

impl Writer {
    /// Creates a writer at the top-left corner with the given colors.
    #[allow(dead_code)] // useful for writing at a fixed screen position
    pub fn new(fg: Color, bg: Color) -> Self {
        Self {
            row: 0,
            col: 0,
            attr: color_code(fg, bg),
        }
    }

    /// Changes the color used for text written after this call.
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.attr = color_code(fg, bg);
    }

    /// Writes one byte at the current position and moves forward.
    /// `\n` starts a new line; other unprintable bytes are shown as a block.
    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.new_line();
            return;
        }
        // Printable ASCII is shown as-is; anything else as a block character.
        let ch = match byte {
            0x20..=0x7e => byte,
            _ => 0xfe,
        };
        write_cell(self.row * WIDTH + self.col, ch, self.attr);
        self.col += 1;
        if self.col >= WIDTH {
            self.new_line();
        }
    }

    /// Writes every byte in `bytes`, then moves the hardware cursor.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
        set_cursor(self.row, self.col);
    }

    /// Moves to the start of the next line, scrolling if already on the last line.
    fn new_line(&mut self) {
        self.col = 0;
        if self.row + 1 >= HEIGHT {
            self.scroll();
        } else {
            self.row += 1;
        }
    }

    /// Moves every row up by one and clears the bottom row.
    fn scroll(&mut self) {
        for row in 0..HEIGHT - 1 {
            for col in 0..WIDTH {
                let cell = row * WIDTH + col;
                let next_cell = (row + 1) * WIDTH + col;
                let (ch, attr) = read_cell(next_cell);
                write_cell(cell, ch, attr);
            }
        }
        self.clear_row(HEIGHT - 1);
    }

    /// Fills `row` with spaces in the current color.
    fn clear_row(&mut self, row: usize) {
        for col in 0..WIDTH {
            write_cell(row * WIDTH + col, b' ', BLANK_ATTR);
        }
    }

    /// Creates a writer at the current hardware cursor position with the given colors.
    pub fn at_cursor(fg: Color, bg: Color) -> Self {
        let (row, col) = get_cursor();
        let mut w = Self {
            row,
            col,
            attr: color_code(fg, bg),
        };
        if w.row >= HEIGHT {
            w.row = HEIGHT - 1;
        }
        if w.col >= WIDTH {
            w.col = WIDTH - 1;
        }
        if w.col != 0 {
            w.new_line();
        }
        w
    }
}
