use proto123_theme::{CapabilityProfile, GlyphSet, ThemeId};

#[test]
fn capability_profile_detects_env_table() {
    let cases = [
        (
            Some("linux"),
            None,
            CapabilityProfile::Console,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("linux"),
            Some("truecolor"),
            CapabilityProfile::Console,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("xterm"),
            Some("truecolor"),
            CapabilityProfile::Full,
            ThemeId::HighContrast,
            GlyphSet::Box,
        ),
        (
            Some("xterm"),
            Some("24bit"),
            CapabilityProfile::Full,
            ThemeId::HighContrast,
            GlyphSet::Box,
        ),
        (
            Some("xterm-256color"),
            None,
            CapabilityProfile::Full,
            ThemeId::HighContrast,
            GlyphSet::Box,
        ),
        (
            Some("tmux-256color"),
            None,
            CapabilityProfile::Full,
            ThemeId::HighContrast,
            GlyphSet::Box,
        ),
        (
            Some("xterm"),
            None,
            CapabilityProfile::Ansi16,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("screen"),
            None,
            CapabilityProfile::Ansi16,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("tmux"),
            None,
            CapabilityProfile::Ansi16,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("vt100"),
            None,
            CapabilityProfile::Ansi16,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("vt220"),
            None,
            CapabilityProfile::Ansi16,
            ThemeId::Console16,
            GlyphSet::Box,
        ),
        (
            Some("dumb"),
            None,
            CapabilityProfile::Conservative,
            ThemeId::Console16,
            GlyphSet::Ascii,
        ),
        (
            Some(""),
            None,
            CapabilityProfile::Conservative,
            ThemeId::Console16,
            GlyphSet::Ascii,
        ),
        (
            None,
            None,
            CapabilityProfile::Conservative,
            ThemeId::Console16,
            GlyphSet::Ascii,
        ),
    ];

    for (term, ct, expected_profile, expected_theme, expected_glyphs) in cases {
        let profile = CapabilityProfile::detect(term, ct);
        assert_eq!(profile, expected_profile, "term={term:?}, colorterm={ct:?}");
        assert_eq!(profile.default_theme(), expected_theme);
        assert_eq!(profile.default_glyphs(), expected_glyphs);
        let theme = profile.theme();
        assert_eq!(theme.id, expected_theme);
        assert_eq!(theme.glyphs, expected_glyphs);
    }
}
