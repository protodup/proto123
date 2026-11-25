//! DOM event primitives → `RawEvent` (contracts/raw-input.md web table). No `web-sys` types
//! here, so this compiles and tests on the native target.

use proto123_theme::GridMetrics;
use proto123_tui::{Button, CellMetrics, Key, Mods, Pos, RawEvent, RawKind, pixel_to_cell};

pub fn mods_from_bits(bits: u8) -> Mods {
    Mods {
        shift: bits & 1 != 0,
        ctrl: bits & 2 != 0,
        alt: bits & 4 != 0,
        meta: bits & 8 != 0,
    }
}

/// `keydown` → `Key`. `key` is the DOM `KeyboardEvent.key` value; `Shift+Tab` becomes `BackTab`.
pub fn keydown(
    key: &str,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
    t_ms: u64,
) -> Option<RawEvent> {
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
        kind: RawKind::Key {
            key: mapped,
            mods: Mods {
                shift,
                ctrl,
                alt,
                meta,
            },
        },
    })
}

/// DOM pointer event fields in CSS pixels (`kind`: 0 down, 1 move, 2 up, 3 leave).
#[derive(Clone, Copy, Debug)]
pub struct DomPointer {
    pub kind: u8,
    pub x: f64,
    pub y: f64,
    pub button: i16,
    pub buttons: u16,
    pub mods_bits: u8,
}

/// DOM `pointerdown`/`pointermove`/`pointerup`/`pointerleave` → `RawEvent` (contracts/wasm-bridge.md).
pub fn pointer(
    p: DomPointer,
    t_ms: u64,
    metrics: &CellMetrics,
    cols: u16,
    rows: u16,
) -> Option<RawEvent> {
    if p.kind == 3 {
        return Some(RawEvent {
            t_ms,
            kind: RawKind::PointerLeave,
        });
    }
    let (col, row) = pixel_to_cell(p.x, p.y, metrics, cols, rows)?;
    let at = Pos::Cell { col, row };
    let mods = mods_from_bits(p.mods_bits);
    let btn = match p.button {
        1 => Button::Middle,
        2 => Button::Right,
        _ => Button::Left,
    };
    let raw_kind = match p.kind {
        0 => RawKind::PointerDown {
            at,
            button: btn,
            mods,
        },
        1 => RawKind::PointerMove {
            at,
            buttons_down: p.buttons & 1 != 0,
            mods,
        },
        2 => RawKind::PointerUp {
            at,
            button: btn,
            mods,
        },
        _ => return None,
    };
    Some(RawEvent {
        t_ms,
        kind: raw_kind,
    })
}

/// DOM `wheel` → `Wheel`, collapsing delta sign to `±1` notch per event (contracts/raw-input.md).
pub fn wheel(dx: f64, dy: f64, mods_bits: u8, t_ms: u64) -> Option<RawEvent> {
    let notch = |v: f64| -> i32 {
        if v > 0.0 {
            1
        } else if v < 0.0 {
            -1
        } else {
            0
        }
    };
    let nx = notch(dx);
    let ny = notch(dy);
    if nx == 0 && ny == 0 {
        return None;
    }
    Some(RawEvent {
        t_ms,
        kind: RawKind::Wheel {
            dx: nx,
            dy: ny,
            mods: mods_from_bits(mods_bits),
        },
    })
}

/// Canvas size change → `Resize` in terminal cells, computed from the given cell metrics.
pub fn resize_with_metrics(metrics: &CellMetrics, css_w: f64, css_h: f64, t_ms: u64) -> RawEvent {
    let (cols, rows) = metrics.grid_size(css_w, css_h);
    RawEvent {
        t_ms,
        kind: RawKind::Resize {
            cols: u16::try_from(cols).unwrap_or(u16::MAX),
            rows: u16::try_from(rows).unwrap_or(u16::MAX),
        },
    }
}

/// Canvas size change → `Resize` in terminal cells, computed from the default theme metrics.
pub fn resize(css_w: f64, css_h: f64, dpr: f64, t_ms: u64) -> RawEvent {
    let metrics = CellMetrics::from_theme(&GridMetrics::DEFAULT, dpr);
    resize_with_metrics(&metrics, css_w, css_h, t_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_tab_is_back_tab_and_named_keys_map() {
        let back = keydown("Tab", true, false, false, false, 1).map(|e| e.kind);
        assert_eq!(
            back,
            Some(RawKind::Key {
                key: Key::BackTab,
                mods: Mods {
                    shift: true,
                    ..Mods::default()
                }
            })
        );
        let left = keydown("ArrowLeft", false, false, false, false, 1).map(|e| e.kind);
        assert_eq!(
            left,
            Some(RawKind::Key {
                key: Key::Left,
                mods: Mods::default()
            })
        );
        let f2 = keydown("F2", false, false, false, false, 1).map(|e| e.kind);
        assert_eq!(
            f2,
            Some(RawKind::Key {
                key: Key::F(2),
                mods: Mods::default()
            })
        );
    }

    #[test]
    fn single_characters_pass_through_and_multi_char_names_are_other() {
        let c = keydown("é", false, false, false, false, 0).map(|e| e.kind);
        assert_eq!(
            c,
            Some(RawKind::Key {
                key: Key::Char('é'),
                mods: Mods::default()
            })
        );
        let other = keydown("Dead", false, false, false, false, 0).map(|e| e.kind);
        assert_eq!(
            other,
            Some(RawKind::Key {
                key: Key::Other,
                mods: Mods::default()
            })
        );
    }

    #[test]
    fn canvas_resize_becomes_cell_counts_from_theme_metrics() {
        // 800×480 CSS px at 10×19 cells with a 5 px origin: 79 × 25 cells.
        assert_eq!(
            resize(800.0, 480.0, 1.0, 0).kind,
            RawKind::Resize { cols: 79, rows: 25 }
        );
    }

    #[test]
    fn pointer_and_wheel_collapse_deltas_and_map_pixels() {
        let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
        let p = pointer(
            DomPointer {
                kind: 0,
                x: 15.0,
                y: 24.0,
                button: 0,
                buttons: 1,
                mods_bits: 1,
            },
            12,
            &m,
            80,
            24,
        )
        .unwrap();
        assert_eq!(
            p,
            RawEvent {
                t_ms: 12,
                kind: RawKind::PointerDown {
                    at: Pos::Cell { col: 1, row: 1 },
                    button: Button::Left,
                    mods: Mods {
                        shift: true,
                        ..Mods::default()
                    },
                },
            }
        );
        assert_eq!(
            pointer(
                DomPointer {
                    kind: 3,
                    x: 0.0,
                    y: 0.0,
                    button: 0,
                    buttons: 0,
                    mods_bits: 0
                },
                13,
                &m,
                80,
                24,
            )
            .map(|e| e.kind),
            Some(RawKind::PointerLeave)
        );
        assert_eq!(
            wheel(0.0, 120.0, 1, 14).map(|e| e.kind),
            Some(RawKind::Wheel {
                dx: 0,
                dy: 1,
                mods: Mods {
                    shift: true,
                    ..Mods::default()
                },
            })
        );
    }
}
