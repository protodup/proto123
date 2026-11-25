//! DOM event primitives → `RawEvent` (contracts/raw-input.md web table). No `web-sys` types
//! here, so this compiles and tests on the native target.

use proto123_theme::GridMetrics;
use proto123_tui::{CellMetrics, Key, Mods, RawEvent, RawKind};

/// `keydown` → `Key`. `key` is the DOM `KeyboardEvent.key` value; `Shift+Tab` becomes `BackTab`.
pub fn keydown(key: &str, shift: bool, ctrl: bool, alt: bool, meta: bool, t_ms: u64) -> Option<RawEvent> {
    let mapped = match key {
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "Enter" => Key::Enter,
        "Escape" => Key::Esc,
        "Tab" if shift => Key::BackTab,
        "Tab" => Key::Tab,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        other => {
            if let Some(n) = other.strip_prefix('F').and_then(|d| d.parse::<u8>().ok()) {
                Key::F(n)
            } else {
                let mut chars = other.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Key::Char(c),
                    _ => Key::Other,
                }
            }
        }
    };
    Some(RawEvent {
        t_ms,
        kind: RawKind::Key { key: mapped, mods: Mods { shift, ctrl, alt, meta } },
    })
}

/// Canvas size change → `Resize` in terminal cells, computed from the theme's cell metrics.
pub fn resize(css_w: f64, css_h: f64, dpr: f64, t_ms: u64) -> RawEvent {
    let metrics = CellMetrics::from_theme(&GridMetrics::DEFAULT, dpr);
    let (cols, rows) = metrics.grid_size(css_w, css_h);
    RawEvent {
        t_ms,
        kind: RawKind::Resize {
            cols: u16::try_from(cols).unwrap_or(u16::MAX),
            rows: u16::try_from(rows).unwrap_or(u16::MAX),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_tab_is_back_tab_and_named_keys_map() {
        let back = keydown("Tab", true, false, false, false, 1).map(|e| e.kind);
        assert_eq!(back, Some(RawKind::Key { key: Key::BackTab, mods: Mods { shift: true, ..Mods::default() } }));
        let left = keydown("ArrowLeft", false, false, false, false, 1).map(|e| e.kind);
        assert_eq!(left, Some(RawKind::Key { key: Key::Left, mods: Mods::default() }));
        let f2 = keydown("F2", false, false, false, false, 1).map(|e| e.kind);
        assert_eq!(f2, Some(RawKind::Key { key: Key::F(2), mods: Mods::default() }));
    }

    #[test]
    fn single_characters_pass_through_and_multi_char_names_are_other() {
        let c = keydown("é", false, false, false, false, 0).map(|e| e.kind);
        assert_eq!(c, Some(RawKind::Key { key: Key::Char('é'), mods: Mods::default() }));
        let other = keydown("Dead", false, false, false, false, 0).map(|e| e.kind);
        assert_eq!(other, Some(RawKind::Key { key: Key::Other, mods: Mods::default() }));
    }

    #[test]
    fn canvas_resize_becomes_cell_counts_from_theme_metrics() {
        // 800×480 CSS px at 10×19 cells with a 5 px origin: 79 × 25 cells.
        assert_eq!(resize(800.0, 480.0, 1.0, 0).kind, RawKind::Resize { cols: 79, rows: 25 });
    }
}
