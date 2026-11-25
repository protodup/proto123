use crate::{Ansi16, Color, GlyphSet, GridMetrics, Pair, Theme, ThemeId, ThemeTokens};

const WHITE: Color = Color::Rgb(255, 255, 255);
const BLACK: Color = Color::Rgb(0, 0, 0);
const YELLOW: Color = Color::Rgb(255, 255, 0);
const CYAN: Color = Color::Rgb(0, 255, 255);

const ANSI_WHITE: Color = Color::Ansi16(Ansi16::White);
const ANSI_BLACK: Color = Color::Ansi16(Ansi16::Black);
const ANSI_LIGHT_YELLOW: Color = Color::Ansi16(Ansi16::LightYellow);
const ANSI_LIGHT_CYAN: Color = Color::Ansi16(Ansi16::LightCyan);

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

/// `console16`: standard ANSI-16 colors only, no bold/underline, single-line box glyphs (FR-048).
pub fn console16() -> Theme {
    Theme {
        id: ThemeId::Console16,
        tokens: ThemeTokens {
            grid: pair(ANSI_WHITE, ANSI_BLACK),
            header: pair(ANSI_LIGHT_CYAN, ANSI_BLACK),
            active_cell: pair(ANSI_BLACK, ANSI_LIGHT_YELLOW),
            selection: pair(ANSI_BLACK, ANSI_LIGHT_CYAN),
            menu: pair(ANSI_WHITE, ANSI_BLACK),
            menu_highlight: pair(ANSI_BLACK, ANSI_WHITE),
            control_line: pair(ANSI_WHITE, ANSI_BLACK),
            status: pair(ANSI_WHITE, ANSI_BLACK),
            badge: pair(ANSI_BLACK, ANSI_LIGHT_YELLOW),
            error: pair(ANSI_LIGHT_YELLOW, ANSI_BLACK),
            border: pair(ANSI_WHITE, ANSI_BLACK),
        },
        glyphs: GlyphSet::Box,
        allow_bold: false,
        metrics: GridMetrics::DEFAULT,
    }
}

/// `mono`: strictly black and white with ASCII glyphs (FR-034).
pub fn mono() -> Theme {
    Theme {
        id: ThemeId::Mono,
        tokens: ThemeTokens {
            grid: pair(WHITE, BLACK),
            header: pair(WHITE, BLACK),
            active_cell: pair(BLACK, WHITE),
            selection: pair(BLACK, WHITE),
            menu: pair(WHITE, BLACK),
            menu_highlight: pair(BLACK, WHITE),
            control_line: pair(WHITE, BLACK),
            status: pair(WHITE, BLACK),
            badge: pair(BLACK, WHITE),
            error: pair(WHITE, BLACK),
            border: pair(WHITE, BLACK),
        },
        glyphs: GlyphSet::Ascii,
        allow_bold: false,
        metrics: GridMetrics::DEFAULT,
    }
}

/// Returns the built-in theme for `id`.
pub fn for_id(id: ThemeId) -> Theme {
    match id {
        ThemeId::HighContrast => high_contrast(),
        ThemeId::Console16 => console16(),
        ThemeId::Mono => mono(),
    }
}
