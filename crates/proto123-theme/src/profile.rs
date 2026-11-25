//! Terminal capability profile detection (research R5, FR-047, FR-048).

use crate::{GlyphSet, Theme, ThemeId, for_id};

/// Terminal color and glyph capability profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityProfile {
    Full,
    Ansi16,
    Console,
    Conservative,
}

impl CapabilityProfile {
    /// Detects the terminal profile from `TERM` and `COLORTERM` environment values.
    pub fn detect(term: Option<&str>, colorterm: Option<&str>) -> Self {
        let term_val = term.unwrap_or("").trim();
        let term_lower = term_val.to_ascii_lowercase();
        let ct_lower = colorterm.unwrap_or("").trim().to_ascii_lowercase();

        if term_lower == "linux" {
            return Self::Console;
        }
        if matches!(ct_lower.as_str(), "truecolor" | "24bit") {
            return Self::Full;
        }
        if term_lower.contains("256color") {
            return Self::Full;
        }
        if term_lower.starts_with("xterm")
            || term_lower.starts_with("screen")
            || term_lower.starts_with("tmux")
            || term_lower.starts_with("vt")
        {
            return Self::Ansi16;
        }
        Self::Conservative
    }

    pub fn default_theme(self) -> ThemeId {
        match self {
            Self::Full => ThemeId::HighContrast,
            Self::Ansi16 | Self::Console | Self::Conservative => ThemeId::Console16,
        }
    }

    pub fn default_glyphs(self) -> GlyphSet {
        match self {
            Self::Conservative => GlyphSet::Ascii,
            Self::Full | Self::Ansi16 | Self::Console => GlyphSet::Box,
        }
    }

    pub fn theme(self) -> Theme {
        let mut t = for_id(self.default_theme());
        t.glyphs = self.default_glyphs();
        t
    }
}
