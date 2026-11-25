//! Theme tokens → ratatui styles (research R6). ANSI colors map to named ratatui colors so
//! the terminal's own palette applies.

use proto123_theme::{Ansi16, Color, Pair};
use ratatui::style::{Color as TuiColor, Style};

pub(crate) fn color(c: Color) -> TuiColor {
    match c {
        Color::Rgb(r, g, b) => TuiColor::Rgb(r, g, b),
        Color::Ansi16(a) => match a {
            Ansi16::Black => TuiColor::Black,
            Ansi16::Red => TuiColor::Red,
            Ansi16::Green => TuiColor::Green,
            Ansi16::Yellow => TuiColor::Yellow,
            Ansi16::Blue => TuiColor::Blue,
            Ansi16::Magenta => TuiColor::Magenta,
            Ansi16::Cyan => TuiColor::Cyan,
            Ansi16::Gray => TuiColor::Gray,
            Ansi16::DarkGray => TuiColor::DarkGray,
            Ansi16::LightRed => TuiColor::LightRed,
            Ansi16::LightGreen => TuiColor::LightGreen,
            Ansi16::LightYellow => TuiColor::LightYellow,
            Ansi16::LightBlue => TuiColor::LightBlue,
            Ansi16::LightMagenta => TuiColor::LightMagenta,
            Ansi16::LightCyan => TuiColor::LightCyan,
            Ansi16::White => TuiColor::White,
        },
    }
}

pub(crate) fn style(pair: Pair) -> Style {
    Style::default().fg(color(pair.fg)).bg(color(pair.bg))
}
