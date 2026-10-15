use crate::port::Port;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

const CRTC_INDEX: Port = Port::new(0x3d4);
const CRTC_DATA: Port = Port::new(0x3d5);

/// Fills the whole screen with spaces, white on black.
pub fn clear_screen() {
    for y in 0..HEIGHT {
        print_at(y, 0, &[b' '; WIDTH], 0x0f);
    }
}

/// Writes `text` starting at row `y`, column `x`.
/// Text that runs past the end of the row is cut off.
pub fn print_at(y: usize, x: usize, text: &[u8], attr: u8) {
    if y >= HEIGHT {
        return;
    }
    for (i, &ch) in text.iter().enumerate() {
        if x + i >= WIDTH {
            break;
        }
        write_cell(y * WIDTH + x + i, ch, attr);
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
#[allow(dead_code)] // not used yet; the Writer in step 3c will need it
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
