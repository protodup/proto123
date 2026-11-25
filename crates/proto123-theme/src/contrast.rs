//! WCAG 2.1 contrast calculation and the Linux VGA 16-color console palette (research R6, FR-048).

use crate::{Ansi16, Color};

/// Standard Linux virtual console VGA 16-color RGB palette, ordered by [`Ansi16`] variant.
pub const VGA_PALETTE: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // Black
    (170, 0, 0),     // Red
    (0, 170, 0),     // Green
    (170, 85, 0),    // Yellow (brown)
    (0, 0, 170),     // Blue
    (170, 0, 170),   // Magenta
    (0, 170, 170),   // Cyan
    (170, 170, 170), // Gray
    (85, 85, 85),    // DarkGray
    (255, 85, 85),   // LightRed
    (85, 255, 85),   // LightGreen
    (255, 255, 85),  // LightYellow
    (85, 85, 255),   // LightBlue
    (255, 85, 255),  // LightMagenta
    (85, 255, 255),  // LightCyan
    (255, 255, 255), // White
];

pub fn to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Ansi16(a) => match a {
            Ansi16::Black => VGA_PALETTE[0],
            Ansi16::Red => VGA_PALETTE[1],
            Ansi16::Green => VGA_PALETTE[2],
            Ansi16::Yellow => VGA_PALETTE[3],
            Ansi16::Blue => VGA_PALETTE[4],
            Ansi16::Magenta => VGA_PALETTE[5],
            Ansi16::Cyan => VGA_PALETTE[6],
            Ansi16::Gray => VGA_PALETTE[7],
            Ansi16::DarkGray => VGA_PALETTE[8],
            Ansi16::LightRed => VGA_PALETTE[9],
            Ansi16::LightGreen => VGA_PALETTE[10],
            Ansi16::LightYellow => VGA_PALETTE[11],
            Ansi16::LightBlue => VGA_PALETTE[12],
            Ansi16::LightMagenta => VGA_PALETTE[13],
            Ansi16::LightCyan => VGA_PALETTE[14],
            Ansi16::White => VGA_PALETTE[15],
        },
    }
}

fn linearize(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.1 relative luminance in `[0.0, 1.0]`.
pub fn relative_luminance(rgb: (u8, u8, u8)) -> f64 {
    0.2126 * linearize(rgb.0) + 0.7152 * linearize(rgb.1) + 0.0722 * linearize(rgb.2)
}

/// WCAG 2.1 contrast ratio in `[1.0, 21.0]`.
pub fn contrast_ratio(fg: Color, bg: Color) -> f64 {
    let l1 = relative_luminance(to_rgb(fg));
    let l2 = relative_luminance(to_rgb(bg));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}
