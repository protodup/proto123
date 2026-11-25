//! The web target: scenario keys become DOM-shaped `keydown` and canvas-size primitives,
//! which go through the same translation the wasm exports use.

use proto123_testkit::{TargetTranslate, check_target, scenarios};
use proto123_tui::{CellMetrics, Key, Mods, RawEvent};
use proto123_theme::GridMetrics;
use proto123_wasm::translate;

/// A DOM event as the host forwards it.
enum WebEvent {
    KeyDown { key: String, shift: bool },
    /// Canvas size in CSS pixels, from the grid metrics.
    Canvas { css_w: f64, css_h: f64 },
}

struct Web;

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
        WebEvent::KeyDown { key: dom_name(key), shift: mods.shift || key == Key::BackTab }
    }

    fn resize_event(&self, cols: u16, rows: u16) -> WebEvent {
        // The canvas size that exactly fits `cols × rows` cells at the theme metrics.
        let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
        WebEvent::Canvas {
            css_w: m.origin_x + f64::from(cols) * m.cell_w,
            css_h: m.origin_y + f64::from(rows) * m.cell_h,
        }
    }

    fn translate(&self, event: WebEvent, t_ms: u64) -> Option<RawEvent> {
        match event {
            WebEvent::KeyDown { key, shift } => translate::keydown(&key, shift, false, false, false, t_ms),
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
