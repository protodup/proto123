//! The web target: scenario keys become DOM-shaped `keydown` and canvas-size primitives,
//! which go through the same translation the wasm exports use.

use proto123_testkit::{TargetTranslate, check_target, scenarios};
use proto123_theme::GridMetrics;
use proto123_tui::{Button, CellMetrics, Key, Mods, Pos, RawEvent, RawKind};
use proto123_wasm::translate;

/// A DOM event as the host forwards it.
enum WebEvent {
    KeyDown {
        key: String,
        shift: bool,
        ctrl: bool,
    },
    Pointer {
        kind: u8,
        x: f64,
        y: f64,
        button: i16,
        buttons: u16,
        mods: u8,
    },
    Wheel {
        dx: f64,
        dy: f64,
        mods: u8,
    },
    /// Canvas size in CSS pixels, from the grid metrics.
    Canvas {
        css_w: f64,
        css_h: f64,
    },
}

struct Web;

fn mods_bits(mods: Mods) -> u8 {
    (u8::from(mods.shift))
        | (u8::from(mods.ctrl) << 1)
        | (u8::from(mods.alt) << 2)
        | (u8::from(mods.meta) << 3)
}

fn button_num(btn: Button) -> i16 {
    match btn {
        Button::Left => 0,
        Button::Middle => 1,
        Button::Right => 2,
    }
}

fn dom_name(key: Key) -> String {
    match key {
        Key::Char(c) => c.to_string(),
        Key::Enter => "Enter".to_owned(),
        Key::Esc => "Escape".to_owned(),
        Key::Tab | Key::BackTab => "Tab".to_owned(),
        Key::Backspace => "Backspace".to_owned(),
        Key::Delete => "Delete".to_owned(),
        Key::Left => "ArrowLeft".to_owned(),
        Key::Right => "ArrowRight".to_owned(),
        Key::Up => "ArrowUp".to_owned(),
        Key::Down => "ArrowDown".to_owned(),
        Key::Home => "Home".to_owned(),
        Key::End => "End".to_owned(),
        Key::PageUp => "PageUp".to_owned(),
        Key::PageDown => "PageDown".to_owned(),
        Key::F(n) => format!("F{n}"),
        Key::Other => "Unidentified".to_owned(),
    }
}

impl TargetTranslate for Web {
    type Event = WebEvent;

    fn key_event(&self, key: Key, mods: Mods) -> WebEvent {
        WebEvent::KeyDown {
            key: dom_name(key),
            shift: mods.shift || key == Key::BackTab,
            ctrl: mods.ctrl,
        }
    }

    fn resize_event(&self, cols: u16, rows: u16) -> WebEvent {
        // The canvas size that exactly fits `cols × rows` cells at the theme metrics.
        let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
        WebEvent::Canvas {
            css_w: m.origin_x + f64::from(cols) * m.cell_w,
            css_h: m.origin_y + f64::from(rows) * m.cell_h,
        }
    }

    fn pointer_event(&self, kind: RawKind) -> Option<WebEvent> {
        let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
        let to_px = |at: Pos| match at {
            Pos::Cell { col, row } => (
                m.origin_x + f64::from(col) * m.cell_w + 1.0,
                m.origin_y + f64::from(row) * m.cell_h + 1.0,
            ),
            Pos::Pixel { x, y } => (x, y),
        };
        match kind {
            RawKind::PointerDown { at, button, mods } => {
                let (x, y) = to_px(at);
                Some(WebEvent::Pointer {
                    kind: 0,
                    x,
                    y,
                    button: button_num(button),
                    buttons: 1,
                    mods: mods_bits(mods),
                })
            }
            RawKind::PointerMove {
                at,
                buttons_down,
                mods,
            } => {
                let (x, y) = to_px(at);
                Some(WebEvent::Pointer {
                    kind: 1,
                    x,
                    y,
                    button: 0,
                    buttons: u16::from(buttons_down),
                    mods: mods_bits(mods),
                })
            }
            RawKind::PointerUp { at, button, mods } => {
                let (x, y) = to_px(at);
                Some(WebEvent::Pointer {
                    kind: 2,
                    x,
                    y,
                    button: button_num(button),
                    buttons: 0,
                    mods: mods_bits(mods),
                })
            }
            RawKind::PointerLeave => Some(WebEvent::Pointer {
                kind: 3,
                x: 0.0,
                y: 0.0,
                button: 0,
                buttons: 0,
                mods: 0,
            }),
            RawKind::Wheel { dx, dy, mods } => Some(WebEvent::Wheel {
                dx: f64::from(dx) * 100.0,
                dy: f64::from(dy) * 100.0,
                mods: mods_bits(mods),
            }),
            _ => None,
        }
    }

    fn translate(&self, event: WebEvent, t_ms: u64) -> Option<RawEvent> {
        let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
        match event {
            WebEvent::KeyDown { key, shift, ctrl } => {
                translate::keydown(&key, shift, ctrl, false, false, t_ms)
            }
            WebEvent::Pointer {
                kind,
                x,
                y,
                button,
                buttons,
                mods,
            } => translate::pointer(
                translate::DomPointer {
                    kind,
                    x,
                    y,
                    button,
                    buttons,
                    mods_bits: mods,
                },
                t_ms,
                &m,
                80,
                24,
            ),
            WebEvent::Wheel { dx, dy, mods } => translate::wheel(dx, dy, mods, t_ms),
            WebEvent::Canvas { css_w, css_h } => Some(translate::resize(css_w, css_h, 1.0, t_ms)),
        }
    }
}

#[test]
fn every_shared_scenario_replays_through_dom_events() {
    for scenario in scenarios::all() {
        if let Err(message) = check_target(&Web, &scenario) {
            panic!("{message}");
        }
    }
}
