//! Shared test support: a key-script DSL, the scenario runner that feeds raw events through
//! `InputMapper` and `Session::dispatch` (the same path both apps use), and the scenario suite.

pub mod scenarios;

use proto123_coord::CellAddr;
use proto123_engine::{Mode, Session};
use proto123_theme::high_contrast;
use proto123_tui::{Button, InputMapper, Key, LayoutMap, Mods, Pos, RawEvent, RawKind, render};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::collections::BTreeMap;

/// What a scenario must show after its last key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub mode: Mode,
    pub active: String,
    /// Populated cells as `address → displayed value`, sorted by address.
    pub cells: BTreeMap<String, String>,
}

pub struct Scenario {
    pub name: &'static str,
    /// Key script in the `keys` DSL below.
    pub script: &'static str,
    pub expect: Snapshot,
}

/// Parses the key DSL: plain characters type themselves; `<enter>`, `<tab>`, `<btab>`,
/// `<esc>`, `<left>`, `<right>`, `<up>`, `<down>`, `<s-left>`, `<s-right>`, `<s-up>`,
/// `<s-down>`, `<c-z>`, `<c-y>`, `<bs>`, `<del>`, `<home>`, `<end>`, `<pgup>`, `<pgdn>`,
/// `<f2>`, `<f8>` are named keys.
pub fn keys(script: &str) -> Result<Vec<RawKind>, String> {
    let mut out = Vec::new();
    let mut rest = script;
    while let Some(c) = rest.chars().next() {
        if c == '<'
            && let Some(end) = rest.find('>')
        {
            let name = rest.get(1..end).unwrap_or("");
            let shift = Mods {
                shift: true,
                ..Mods::default()
            };
            let ctrl = Mods {
                ctrl: true,
                ..Mods::default()
            };
            let (key, mods) = match name {
                "enter" => (Key::Enter, Mods::default()),
                "tab" => (Key::Tab, Mods::default()),
                "btab" => (Key::BackTab, Mods::default()),
                "esc" => (Key::Esc, Mods::default()),
                "left" => (Key::Left, Mods::default()),
                "right" => (Key::Right, Mods::default()),
                "up" => (Key::Up, Mods::default()),
                "down" => (Key::Down, Mods::default()),
                "s-left" => (Key::Left, shift),
                "s-right" => (Key::Right, shift),
                "s-up" => (Key::Up, shift),
                "s-down" => (Key::Down, shift),
                "c-z" => (Key::Char('z'), ctrl),
                "c-y" => (Key::Char('y'), ctrl),
                "bs" => (Key::Backspace, Mods::default()),
                "del" => (Key::Delete, Mods::default()),
                "home" => (Key::Home, Mods::default()),
                "end" => (Key::End, Mods::default()),
                "pgup" => (Key::PageUp, Mods::default()),
                "pgdn" => (Key::PageDown, Mods::default()),
                "f2" => (Key::F(2), Mods::default()),
                "f8" => (Key::F(8), Mods::default()),
                other if other.starts_with("pdown:") => {
                    let (col, row) = parse_pair(&other[6..], name)?;
                    out.push(RawKind::PointerDown {
                        at: Pos::Cell { col, row },
                        button: Button::Left,
                        mods: Mods::default(),
                    });
                    rest = rest.get(end + 1..).unwrap_or("");
                    continue;
                }
                other if other.starts_with("pmove:") => {
                    let (col, row) = parse_pair(&other[6..], name)?;
                    out.push(RawKind::PointerMove {
                        at: Pos::Cell { col, row },
                        buttons_down: true,
                        mods: Mods::default(),
                    });
                    rest = rest.get(end + 1..).unwrap_or("");
                    continue;
                }
                other if other.starts_with("pup:") => {
                    let (col, row) = parse_pair(&other[4..], name)?;
                    out.push(RawKind::PointerUp {
                        at: Pos::Cell { col, row },
                        button: Button::Left,
                        mods: Mods::default(),
                    });
                    rest = rest.get(end + 1..).unwrap_or("");
                    continue;
                }
                other if other.starts_with("wheel:") => {
                    let (dx, dy) = parse_signed_pair(&other[6..], name)?;
                    out.push(RawKind::Wheel {
                        dx,
                        dy,
                        mods: Mods::default(),
                    });
                    rest = rest.get(end + 1..).unwrap_or("");
                    continue;
                }
                other => return Err(format!("unknown key name {other:?} in scenario script")),
            };
            out.push(RawKind::Key { key, mods });
            rest = rest.get(end + 1..).unwrap_or("");
            continue;
        }
        out.push(RawKind::Key {
            key: Key::Char(c),
            mods: Mods::default(),
        });
        rest = rest.get(c.len_utf8()..).unwrap_or("");
    }
    Ok(out)
}

fn parse_pair(s: &str, name: &str) -> Result<(u16, u16), String> {
    let (a, b) = s
        .split_once(',')
        .ok_or_else(|| format!("bad coordinates in <{name}>"))?;
    let col = a
        .parse::<u16>()
        .map_err(|e| format!("bad col in <{name}>: {e}"))?;
    let row = b
        .parse::<u16>()
        .map_err(|e| format!("bad row in <{name}>: {e}"))?;
    Ok((col, row))
}

fn parse_signed_pair(s: &str, name: &str) -> Result<(i32, i32), String> {
    let (a, b) = s
        .split_once(',')
        .ok_or_else(|| format!("bad deltas in <{name}>"))?;
    let dx = a
        .parse::<i32>()
        .map_err(|e| format!("bad dx in <{name}>: {e}"))?;
    let dy = b
        .parse::<i32>()
        .map_err(|e| format!("bad dy in <{name}>: {e}"))?;
    Ok((dx, dy))
}

/// The DSL's key sequence on an 80×24 terminal: a resize, then the script's keys.
pub fn core_kinds(script: &str) -> Result<Vec<RawKind>, String> {
    let mut kinds = vec![RawKind::Resize { cols: 80, rows: 24 }];
    kinds.extend(keys(script)?);
    Ok(kinds)
}

/// Feeds raw events through `InputMapper` and `Session::dispatch` with an explicit theme.
pub fn replay_with_theme(events: &[RawEvent], theme: proto123_theme::Theme) -> (Session, Snapshot) {
    let mut session = Session::new(theme);
    let mut mapper = InputMapper::new(session.theme().metrics);
    let area = Rect::new(0, 0, 80, 24);
    let mut layout = LayoutMap::default();
    for event in events {
        for command in mapper.map(*event, &layout, &session) {
            session.dispatch(command);
        }
        let mut buf = Buffer::empty(area);
        layout = render(&session, area, &mut buf);
    }
    let snapshot = snapshot_of(&session);
    (session, snapshot)
}

/// Feeds raw events through `InputMapper` and `Session::dispatch` and returns the result.
/// This is the single path every target's events take after translation.
pub fn replay(events: &[RawEvent]) -> (Session, Snapshot) {
    replay_with_theme(events, high_contrast())
}

/// Stamps kinds with increasing monotonic times, as a host would.
pub fn stamp(kinds: &[RawKind]) -> Vec<RawEvent> {
    kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| RawEvent {
            t_ms: (i as u64 + 1) * 10,
            kind: *kind,
        })
        .collect()
}

/// Runs a scenario on the core at 80×24 and returns the final session and snapshot.
pub fn run_core(script: &str) -> Result<(Session, Snapshot), String> {
    Ok(replay(&stamp(&core_kinds(script)?)))
}

/// One target's translation: how its platform events are produced for a DSL key and how
/// the app turns them back into a `RawEvent`. Scenarios replay through this so the
/// translation itself is tested, not just the core.
pub trait TargetTranslate {
    type Event;
    fn key_event(&self, key: Key, mods: Mods) -> Self::Event;
    fn resize_event(&self, cols: u16, rows: u16) -> Self::Event;
    fn pointer_event(&self, kind: RawKind) -> Option<Self::Event> {
        let _ = kind;
        None
    }
    fn translate(&self, event: Self::Event, t_ms: u64) -> Option<RawEvent>;
}

/// Replays a scenario through a target's translation with a given theme and checks the snapshot.
pub fn check_target_with_theme<T: TargetTranslate>(
    target: &T,
    s: &Scenario,
    theme: proto123_theme::Theme,
) -> Result<(), String> {
    let kinds = core_kinds(s.script).map_err(|e| format!("scenario {:?}: {e}", s.name))?;
    let mut events = Vec::new();
    for (i, kind) in kinds.iter().enumerate() {
        let t_ms = (i as u64 + 1) * 10;
        let platform = match kind {
            RawKind::Key { key, mods } => target.key_event(*key, *mods),
            RawKind::Resize { cols, rows } => target.resize_event(*cols, *rows),
            other => target
                .pointer_event(*other)
                .ok_or_else(|| format!("scenario {:?}: unexpected kind {other:?}", s.name))?,
        };
        match target.translate(platform, t_ms) {
            Some(event) => events.push(event),
            None => {
                return Err(format!(
                    "scenario {:?}: target dropped event {kind:?}",
                    s.name
                ));
            }
        }
    }
    let (_, actual) = replay_with_theme(&events, theme);
    if actual == s.expect {
        Ok(())
    } else {
        Err(format!(
            "scenario {:?}: expected {:?}, got {:?}",
            s.name, s.expect, actual
        ))
    }
}

/// Replays a scenario through a target's translation and checks the snapshot.
pub fn check_target<T: TargetTranslate>(target: &T, s: &Scenario) -> Result<(), String> {
    check_target_with_theme(target, s, high_contrast())
}

pub fn snapshot_of(session: &Session) -> Snapshot {
    let mut cells = BTreeMap::new();
    for (addr, value) in populated(session) {
        cells.insert(addr.to_string(), value.to_string());
    }
    Snapshot {
        mode: session.mode(),
        active: session.active().to_string(),
        cells,
    }
}

fn populated(session: &Session) -> Vec<(CellAddr, proto123_value::Value)> {
    let mut out = Vec::new();
    let sheet = session.calc().sheet();
    let range = proto123_coord::CellRange::new(
        CellAddr::ORIGIN,
        CellAddr::new(16_383, 1_048_575).unwrap_or(CellAddr::ORIGIN),
    );
    sheet.for_each_populated_in(range, |addr, cell| out.push((addr, cell.value.clone())));
    out
}

/// Runs a scenario on the core and returns the first mismatch, naming the scenario.
pub fn check_scenario(s: &Scenario) -> Result<(), String> {
    let (_, actual) = run_core(s.script).map_err(|e| format!("scenario {:?}: {e}", s.name))?;
    if actual == s.expect {
        Ok(())
    } else {
        Err(format!(
            "scenario {:?}: expected {:?}, got {:?}",
            s.name, s.expect, actual
        ))
    }
}
