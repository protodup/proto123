use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use proto123_term::translate::translate;
use proto123_testkit::{TargetTranslate, check_target_with_theme, scenarios};
use proto123_theme::CapabilityProfile;
use proto123_tui::{Button, Key, Mods, Pos, RawEvent, RawKind};

struct Native;

fn key_code(key: Key) -> KeyCode {
    match key {
        Key::Char(c) => KeyCode::Char(c),
        Key::Enter => KeyCode::Enter,
        Key::Esc => KeyCode::Esc,
        Key::Tab => KeyCode::Tab,
        Key::BackTab => KeyCode::BackTab,
        Key::Backspace => KeyCode::Backspace,
        Key::Delete => KeyCode::Delete,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::F(n) => KeyCode::F(n),
        Key::Other => KeyCode::Null,
    }
}

impl TargetTranslate for Native {
    type Event = Event;

    fn key_event(&self, key: Key, mods: Mods) -> Event {
        let mut m = KeyModifiers::empty();
        m.set(KeyModifiers::SHIFT, mods.shift);
        m.set(KeyModifiers::CONTROL, mods.ctrl);
        m.set(KeyModifiers::ALT, mods.alt);
        m.set(KeyModifiers::SUPER, mods.meta);
        Event::Key(KeyEvent::new(key_code(key), m))
    }

    fn resize_event(&self, cols: u16, rows: u16) -> Event {
        Event::Resize(cols, rows)
    }

    fn pointer_event(&self, kind: RawKind) -> Option<Event> {
        let ct_mods = |mods: Mods| {
            let mut m = KeyModifiers::empty();
            m.set(KeyModifiers::SHIFT, mods.shift);
            m.set(KeyModifiers::CONTROL, mods.ctrl);
            m.set(KeyModifiers::ALT, mods.alt);
            m.set(KeyModifiers::SUPER, mods.meta);
            m
        };
        let ct_btn = |btn: Button| match btn {
            Button::Left => MouseButton::Left,
            Button::Middle => MouseButton::Middle,
            Button::Right => MouseButton::Right,
        };
        match kind {
            RawKind::PointerDown {
                at: Pos::Cell { col, row },
                button,
                mods,
            } => Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(ct_btn(button)),
                column: col,
                row,
                modifiers: ct_mods(mods),
            })),
            RawKind::PointerMove {
                at: Pos::Cell { col, row },
                buttons_down,
                mods,
            } => Some(Event::Mouse(MouseEvent {
                kind: if buttons_down {
                    MouseEventKind::Drag(MouseButton::Left)
                } else {
                    MouseEventKind::Moved
                },
                column: col,
                row,
                modifiers: ct_mods(mods),
            })),
            RawKind::PointerUp {
                at: Pos::Cell { col, row },
                button,
                mods,
            } => Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(ct_btn(button)),
                column: col,
                row,
                modifiers: ct_mods(mods),
            })),
            RawKind::Wheel { dx, dy, mods } => {
                let kind = if dy > 0 {
                    MouseEventKind::ScrollDown
                } else if dy < 0 {
                    MouseEventKind::ScrollUp
                } else if dx > 0 {
                    MouseEventKind::ScrollRight
                } else {
                    MouseEventKind::ScrollLeft
                };
                Some(Event::Mouse(MouseEvent {
                    kind,
                    column: 0,
                    row: 0,
                    modifiers: ct_mods(mods),
                }))
            }
            _ => None,
        }
    }

    fn translate(&self, event: Event, t_ms: u64) -> Option<RawEvent> {
        translate(event, t_ms)
    }
}

#[test]
fn every_shared_scenario_replays_on_full_and_console_profiles() {
    let full = CapabilityProfile::detect(Some("xterm-256color"), Some("truecolor")).theme();
    let console = CapabilityProfile::detect(Some("linux"), None).theme();
    for scenario in scenarios::all() {
        check_target_with_theme(&Native, &scenario, full).unwrap();
        check_target_with_theme(&Native, &scenario, console).unwrap();
    }
}

#[test]
fn keyboard_only_scenarios_complete_without_mouse_events() {
    let console = CapabilityProfile::detect(Some("linux"), None).theme();
    for scenario in scenarios::all()
        .into_iter()
        .filter(|s| !s.name.starts_with("us4_"))
    {
        let kinds = proto123_testkit::core_kinds(scenario.script).unwrap();
        assert!(
            kinds
                .iter()
                .all(|k| matches!(k, RawKind::Key { .. } | RawKind::Resize { .. })),
            "scenario {} contained pointer event",
            scenario.name
        );
        check_target_with_theme(&Native, &scenario, console).unwrap();
    }
}
