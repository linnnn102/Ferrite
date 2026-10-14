const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

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
