use proto123_theme::{
    Color, GlyphSet, GridMetrics, Theme, ThemeId, VGA_PALETTE, console16, contrast_ratio, for_id,
    high_contrast, mono,
};

fn check_theme_contrast(theme: &Theme) {
    let t = &theme.tokens;
    let text_pairs = [
        ("grid", t.grid),
        ("header", t.header),
        ("selection", t.selection),
        ("menu", t.menu),
        ("control_line", t.control_line),
        ("status", t.status),
        ("badge", t.badge),
        ("error", t.error),
    ];
    for (name, pair) in text_pairs {
        let ratio = contrast_ratio(pair.fg, pair.bg);
        assert!(
            ratio >= 4.5,
            "theme {} token {name} contrast {ratio:.2} < 4.5",
            theme.id.as_str()
        );
    }
    let ui_pairs = [
        ("border", t.border),
        ("active_cell", t.active_cell),
        ("menu_highlight", t.menu_highlight),
    ];
    for (name, pair) in ui_pairs {
        let ratio = contrast_ratio(pair.fg, pair.bg);
        assert!(
            ratio >= 3.0,
            "theme {} token {name} contrast {ratio:.2} < 3.0",
            theme.id.as_str()
        );
    }
}

#[test]
fn all_builtin_themes_meet_wcag_contrast_and_share_grid_metrics() {
    for id in ThemeId::ALL {
        let theme = for_id(id);
        check_theme_contrast(&theme);
        assert_eq!(theme.metrics, GridMetrics::DEFAULT);
        assert_eq!(theme.metrics.font_px, 16);
        assert_eq!(theme.metrics.font_family, "monospace");
        assert_eq!(theme.metrics.cell_px, (10, 19));
        assert_eq!(theme.metrics.origin_px, 5);
    }
}

#[test]
fn console16_uses_only_ansi16_without_bold_against_vga_palette() {
    let theme = console16();
    assert_eq!(VGA_PALETTE.len(), 16);
    assert!(!theme.allow_bold);
    assert_eq!(theme.glyphs, GlyphSet::Box);
    let t = theme.tokens;
    for pair in [
        t.grid,
        t.header,
        t.active_cell,
        t.selection,
        t.menu,
        t.menu_highlight,
        t.control_line,
        t.status,
        t.badge,
        t.error,
        t.border,
    ] {
        assert!(matches!(pair.fg, Color::Ansi16(_)));
        assert!(matches!(pair.bg, Color::Ansi16(_)));
    }
}

#[test]
fn mono_uses_only_black_and_white_with_ascii_glyphs() {
    let theme = mono();
    assert!(!theme.allow_bold);
    assert_eq!(theme.glyphs, GlyphSet::Ascii);
    let is_bw = |c: Color| matches!(c, Color::Rgb(0, 0, 0) | Color::Rgb(255, 255, 255));
    let t = theme.tokens;
    for pair in [
        t.grid,
        t.header,
        t.active_cell,
        t.selection,
        t.menu,
        t.menu_highlight,
        t.control_line,
        t.status,
        t.badge,
        t.error,
        t.border,
    ] {
        assert!(is_bw(pair.fg));
        assert!(is_bw(pair.bg));
    }
}

#[test]
fn high_contrast_has_expected_id_and_cycle() {
    assert_eq!(high_contrast().id, ThemeId::HighContrast);
    assert_eq!(ThemeId::HighContrast.next(), ThemeId::Console16);
    assert_eq!(ThemeId::Console16.next(), ThemeId::Mono);
    assert_eq!(ThemeId::Mono.next(), ThemeId::HighContrast);
}
