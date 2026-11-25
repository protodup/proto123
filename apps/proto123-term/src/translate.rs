//! crossterm event → `RawEvent` (contracts/raw-input.md native table). No interpretation.

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use proto123_tui::{Key, Mods, RawEvent, RawKind};

fn map_key(code: KeyCode) -> Key {
    match code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::F(n) => Key::F(n),
        _ => Key::Other,
    }
}

fn map_mods(m: KeyModifiers) -> Mods {
    Mods {
        shift: m.contains(KeyModifiers::SHIFT),
        ctrl: m.contains(KeyModifiers::CONTROL),
        alt: m.contains(KeyModifiers::ALT),
        meta: m.contains(KeyModifiers::SUPER),
    }
}

/// Translates one terminal event. Key releases are ignored; `t_ms` is supplied by the caller.
pub fn translate(event: Event, t_ms: u64) -> Option<RawEvent> {
    match event {
        Event::Key(k) if k.kind != KeyEventKind::Release => Some(RawEvent {
            t_ms,
            kind: RawKind::Key { key: map_key(k.code), mods: map_mods(k.modifiers) },
        }),
        Event::Resize(cols, rows) => Some(RawEvent { t_ms, kind: RawKind::Resize { cols, rows } }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    #[test]
    fn key_press_maps_with_modifiers_and_releases_are_dropped() {
        let press = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let raw = translate(Event::Key(press), 42).expect("press is forwarded");
        assert_eq!(raw.t_ms, 42);
        assert_eq!(
            raw.kind,
            RawKind::Key { key: Key::Char('q'), mods: Mods { ctrl: true, ..Mods::default() } }
        );

        let release = KeyEvent::new_with_kind(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(translate(Event::Key(release), 0), None);
    }

    #[test]
    fn back_tab_and_resize_map_directly() {
        let back = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(
            translate(Event::Key(back), 0).map(|r| r.kind),
            Some(RawKind::Key { key: Key::BackTab, mods: Mods { shift: true, ..Mods::default() } })
        );
        assert_eq!(
            translate(Event::Resize(80, 24), 0).map(|r| r.kind),
            Some(RawKind::Resize { cols: 80, rows: 24 })
        );
    }
}
