//! L0 theme tokens: colors, glyph sets, grid metrics, and the built-in themes.
//! Nothing here knows about a browser, terminal, or widget library.

mod builtin;

pub use builtin::high_contrast;

/// The 16 ANSI colors (terminals map these to their own palette).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ansi16 {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Ansi16(Ansi16),
}

/// Box-drawing glyphs (`Box`) or plain ASCII (`Ascii`) for every line and border.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphSet {
    Box,
    Ascii,
}

/// A foreground/background pair for one named token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub fg: Color,
    pub bg: Color,
}

/// Named color tokens used by every widget. Renderers never pick literal colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeTokens {
    pub grid: Pair,
    pub header: Pair,
    pub active_cell: Pair,
    pub selection: Pair,
    pub menu: Pair,
    pub menu_highlight: Pair,
    pub control_line: Pair,
    pub status: Pair,
    pub badge: Pair,
    pub error: Pair,
    pub border: Pair,
}

/// Character-cell metrics. Web font tokens follow what Ratzilla 0.3 draws (research R2);
/// native targets ignore them because the terminal emulator owns the font (FR-050).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridMetrics {
    /// Width of one grid column in characters (FR-006).
    pub col_width: u16,
    /// Width of the row-number gutter in characters.
    pub row_header_width: u16,
    pub min_cols: u16,
    pub min_rows: u16,
    pub font_px: u16,
    pub font_family: &'static str,
    /// Cell size in CSS pixels `(width, height)`.
    pub cell_px: (u16, u16),
    pub origin_px: u16,
}

impl GridMetrics {
    pub const DEFAULT: Self = Self {
        col_width: 9,
        row_header_width: 5,
        min_cols: 80,
        min_rows: 24,
        font_px: 16,
        font_family: "monospace",
        cell_px: (10, 19),
        origin_px: 5,
    };
}

/// Built-in theme identifiers in cycle order (View → Theme cycles through them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeId {
    HighContrast,
    Console16,
    Mono,
}

impl ThemeId {
    pub const ALL: [ThemeId; 3] = [Self::HighContrast, Self::Console16, Self::Mono];

    /// The next theme in cycle order; wraps after the last.
    pub fn next(self) -> Self {
        match self {
            Self::HighContrast => Self::Console16,
            Self::Console16 => Self::Mono,
            Self::Mono => Self::HighContrast,
        }
    }
}

/// A complete theme: tokens plus glyph and style policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub id: ThemeId,
    pub tokens: ThemeTokens,
    pub glyphs: GlyphSet,
    /// When false, renderers must not emit BOLD or UNDERLINED modifiers.
    pub allow_bold: bool,
    pub metrics: GridMetrics,
}
