#[cfg(test)]
mod contrast;
#[cfg(test)]
mod profile;

use proto123_theme::{Color, GridMetrics, ThemeId, high_contrast};

#[test]
fn high_contrast_carries_the_shared_grid_metrics() {
    let theme = high_contrast();
    assert_eq!(theme.id, ThemeId::HighContrast);
    assert_eq!(theme.metrics, GridMetrics::DEFAULT);
    assert!(matches!(theme.tokens.grid.fg, Color::Rgb(..)));
}

#[test]
fn theme_cycle_wraps_after_last() {
    assert_eq!(ThemeId::HighContrast.next(), ThemeId::Console16);
    assert_eq!(ThemeId::Console16.next(), ThemeId::Mono);
    assert_eq!(ThemeId::Mono.next(), ThemeId::HighContrast);
}
