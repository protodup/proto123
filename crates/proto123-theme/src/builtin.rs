use crate::{Color, GlyphSet, GridMetrics, Pair, Theme, ThemeId, ThemeTokens};

const WHITE: Color = Color::Rgb(255, 255, 255);
const BLACK: Color = Color::Rgb(0, 0, 0);
const YELLOW: Color = Color::Rgb(255, 255, 0);
const CYAN: Color = Color::Rgb(0, 255, 255);

const fn pair(fg: Color, bg: Color) -> Pair {
    Pair { fg, bg }
}

/// `high-contrast`: white on black, yellow active cell, box-drawing glyphs.
pub fn high_contrast() -> Theme {
    Theme {
        id: ThemeId::HighContrast,
        tokens: ThemeTokens {
            grid: pair(WHITE, BLACK),
            header: pair(CYAN, BLACK),
            active_cell: pair(BLACK, YELLOW),
            selection: pair(BLACK, CYAN),
            menu: pair(WHITE, BLACK),
            menu_highlight: pair(BLACK, WHITE),
            control_line: pair(WHITE, BLACK),
            status: pair(WHITE, BLACK),
            badge: pair(BLACK, YELLOW),
            error: pair(YELLOW, BLACK),
            border: pair(WHITE, BLACK),
        },
        glyphs: GlyphSet::Box,
        allow_bold: true,
        metrics: GridMetrics::DEFAULT,
    }
}
