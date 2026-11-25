//! `InputMapper`: raw key events → engine commands. Gesture interpretation (mouse, double
//! click, drag) is added with US4; the key path is complete here for the READY/VALUE/LABEL/EDIT modes.

use crate::layout::{grid_area, meets_minimum};
use crate::layout_map::LayoutMap;
use crate::raw::{Key, Mods, RawEvent, RawKind};
use proto123_engine::{CaretKey, Command, HomeKind, Mode, Session, Then};
use proto123_theme::GridMetrics;

#[derive(Clone, Debug)]
pub struct InputMapper {
    metrics: GridMetrics,
}

impl InputMapper {
    pub fn new(metrics: GridMetrics) -> Self {
        Self { metrics }
    }

    /// Maps one raw event to zero or more commands. Below the minimum size the layout map is
    /// empty and only resizes are honoured, so no key can change the workbook (FR-033).
    pub fn map(&mut self, event: RawEvent, layout: &LayoutMap, session: &Session) -> Vec<Command> {
        match event.kind {
            RawKind::Resize { cols, rows } => {
                if meets_minimum(cols, rows, &self.metrics) {
                    let area = grid_area(cols, rows, &self.metrics);
                    vec![Command::SessionResize { cols: area.cols, rows: area.rows }]
                } else {
                    Vec::new()
                }
            }
            RawKind::Key { key, mods } if !layout.is_empty() => map_key(key, mods, session.mode()),
            _ => Vec::new(),
        }
    }
}

fn plain(mods: Mods) -> bool {
    !(mods.ctrl || mods.alt || mods.meta)
}

fn map_key(key: Key, mods: Mods, mode: Mode) -> Vec<Command> {
    let step = |dcol: i64, drow: i64| vec![Command::NavMoveRelative { dcol, drow }];
    let commit = |dcol: i64, drow: i64| vec![Command::CellCommitEdit(Then::Move { dcol, drow })];
    let caret = |k: CaretKey| vec![Command::CellEditCaret(k)];
    match mode {
        Mode::Ready => match key {
            Key::Char(c) if plain(mods) => vec![Command::CellBeginEntry(c)],
            Key::Left => step(-1, 0),
            Key::Right => step(1, 0),
            Key::Up => step(0, -1),
            Key::Down => step(0, 1),
            Key::Tab if !mods.shift => step(1, 0),
            Key::BackTab => step(-1, 0),
            Key::Tab => step(-1, 0),
            Key::Enter if mods.shift => step(0, -1),
            Key::Enter => step(0, 1),
            Key::PageUp => vec![Command::NavPageScroll { horizontal: mods.alt, dir: -1 }],
            Key::PageDown => vec![Command::NavPageScroll { horizontal: mods.alt, dir: 1 }],
            Key::Home if mods.ctrl => vec![Command::NavJumpHome(HomeKind::A1)],
            Key::Home => vec![Command::NavJumpHome(HomeKind::RowStart)],
            Key::F(2) => vec![Command::CellBeginEdit],
            _ => Vec::new(),
        },
        Mode::Value | Mode::Label => match key {
            Key::Char(c) if plain(mods) => vec![Command::CellEditInput(c)],
            Key::Left => commit(-1, 0),
            Key::Right => commit(1, 0),
            Key::Up => commit(0, -1),
            Key::Down => commit(0, 1),
            Key::Enter if mods.shift => commit(0, -1),
            Key::Enter => commit(0, 1),
            Key::Tab if mods.shift => commit(-1, 0),
            Key::BackTab => commit(-1, 0),
            Key::Tab => commit(1, 0),
            Key::Home => caret(CaretKey::Home),
            Key::End => caret(CaretKey::End),
            Key::Backspace => caret(CaretKey::Backspace),
            Key::Delete => caret(CaretKey::Delete),
            Key::Esc => vec![Command::CellCancelEdit],
            _ => Vec::new(),
        },
        Mode::Edit => match key {
            Key::Char(c) if plain(mods) => vec![Command::CellEditInput(c)],
            Key::Left => caret(CaretKey::Left),
            Key::Right => caret(CaretKey::Right),
            Key::Home => caret(CaretKey::Home),
            Key::End => caret(CaretKey::End),
            Key::Backspace => caret(CaretKey::Backspace),
            Key::Delete => caret(CaretKey::Delete),
            Key::Up => commit(0, -1),
            Key::Down => commit(0, 1),
            Key::Enter if mods.shift => commit(0, -1),
            Key::Enter => commit(0, 1),
            Key::Tab if mods.shift => commit(-1, 0),
            Key::BackTab => commit(-1, 0),
            Key::Tab => commit(1, 0),
            Key::Esc => vec![Command::CellCancelEdit],
            _ => Vec::new(),
        },
    }
}
