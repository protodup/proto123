//! `InputMapper`: raw key, pointer, and wheel events → engine commands (FR-028..031).

use crate::layout::{grid_area, meets_minimum};
use crate::layout_map::{EdgeDir, HitTarget, LayoutMap};
use crate::raw::{Button, Key, Mods, Pos, RawEvent, RawKind};
use proto123_coord::{CellAddr, MAX_COL, MAX_ROW};
use proto123_engine::{CaretKey, Command, HomeKind, Mode, Prompt, PromptButton, Session, Then};
use proto123_theme::GridMetrics;

const DOUBLE_CLICK_MS: u64 = 500;
const WHEEL_ROWS_PER_NOTCH: i64 = 3;

#[derive(Clone, Copy)]
struct PointerClick {
    col: u16,
    row: u16,
    button: Button,
    mods: Mods,
    t_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragKind {
    Grid,
    Point { anchor: CellAddr },
    Prompt { anchor: CellAddr },
}

#[derive(Clone, Debug)]
pub struct InputMapper {
    metrics: GridMetrics,
    last_press: Option<(CellAddr, u64)>,
    drag: Option<DragKind>,
    hover: Option<HitTarget>,
}

impl InputMapper {
    pub fn new(metrics: GridMetrics) -> Self {
        Self {
            metrics,
            last_press: None,
            drag: None,
            hover: None,
        }
    }

    /// Maps one raw event to zero or more commands. Below the minimum size the layout map is
    /// empty and only resizes are honoured, so no input can change the workbook (FR-033).
    pub fn map(&mut self, event: RawEvent, layout: &LayoutMap, session: &Session) -> Vec<Command> {
        match event.kind {
            RawKind::Resize { cols, rows } => {
                self.drag = None;
                self.hover = None;
                if meets_minimum(cols, rows, &self.metrics) {
                    let area = grid_area(cols, rows, &self.metrics);
                    vec![Command::SessionResize {
                        cols: area.cols,
                        rows: area.rows,
                    }]
                } else {
                    Vec::new()
                }
            }
            RawKind::Key { key, mods } if !layout.is_empty() => map_key(key, mods, session),
            RawKind::PointerDown {
                at: Pos::Cell { col, row },
                button,
                mods,
            } if !layout.is_empty() => self.map_pointer_down(
                PointerClick {
                    col,
                    row,
                    button,
                    mods,
                    t_ms: event.t_ms,
                },
                layout,
                session,
            ),
            RawKind::PointerMove {
                at: Pos::Cell { col, row },
                buttons_down,
                ..
            } if !layout.is_empty() => {
                self.map_pointer_move(col, row, buttons_down, layout, session)
            }
            RawKind::PointerUp { .. } => {
                self.drag = None;
                Vec::new()
            }
            RawKind::PointerLeave => {
                self.drag = None;
                let was_menu = matches!(self.hover, Some(HitTarget::MenuItem { .. }));
                self.hover = None;
                if was_menu {
                    vec![Command::MenuHover(None)]
                } else {
                    Vec::new()
                }
            }
            RawKind::Wheel { dx, dy, mods } if !layout.is_empty() => map_wheel(dx, dy, mods),
            _ => Vec::new(),
        }
    }

    fn map_pointer_down(
        &mut self,
        click: PointerClick,
        layout: &LayoutMap,
        session: &Session,
    ) -> Vec<Command> {
        let PointerClick {
            col,
            row,
            button,
            mods,
            t_ms,
        } = click;
        if button != Button::Left {
            return Vec::new();
        }
        let target = layout.hit(col, row);
        if session.mode() == Mode::Menu && !matches!(target, HitTarget::MenuItem { .. }) {
            self.drag = None;
            self.last_press = None;
            return vec![Command::MenuClose];
        }
        match target {
            HitTarget::GridCell(addr) => {
                if session.mode() == Mode::Ready
                    && self.last_press.is_some_and(|(prev_addr, prev_t)| {
                        prev_addr == addr && t_ms.saturating_sub(prev_t) <= DOUBLE_CLICK_MS
                    })
                {
                    self.last_press = None;
                    self.drag = None;
                    return vec![Command::CellBeginEditAt {
                        addr: Some(addr),
                        caret: None,
                    }];
                }
                self.last_press = Some((addr, t_ms));
                match session.mode() {
                    Mode::Ready => {
                        self.drag = Some(DragKind::Grid);
                        if mods.shift {
                            vec![Command::NavExtendSelection(addr)]
                        } else {
                            vec![Command::NavSelectCell(addr)]
                        }
                    }
                    Mode::Value | Mode::Edit | Mode::Point => {
                        if session.is_point_eligible() || session.mode() == Mode::Point {
                            self.drag = Some(DragKind::Point { anchor: addr });
                            vec![Command::PointSetReference {
                                anchor: None,
                                cursor: addr,
                            }]
                        } else {
                            self.drag = None;
                            vec![Command::CellCommitEdit(Then::Select(addr))]
                        }
                    }
                    Mode::Label => {
                        self.drag = None;
                        vec![Command::CellCommitEdit(Then::Select(addr))]
                    }
                    Mode::Dialog => {
                        if matches!(
                            session.prompt(),
                            Some(Prompt::RangeSource { .. } | Prompt::RangeDest { .. })
                        ) {
                            self.drag = Some(DragKind::Prompt { anchor: addr });
                            vec![Command::PointSetReference {
                                anchor: None,
                                cursor: addr,
                            }]
                        } else {
                            Vec::new()
                        }
                    }
                    Mode::Menu => Vec::new(),
                }
            }
            HitTarget::FormulaBar { char_index } => {
                self.drag = None;
                self.last_press = None;
                vec![Command::CellBeginEditAt {
                    addr: None,
                    caret: Some(char_index),
                }]
            }
            HitTarget::MenuItem { level, index } => {
                self.drag = None;
                self.last_press = None;
                match session.mode() {
                    Mode::Ready if level == 0 => vec![Command::MenuOpenAt(index)],
                    Mode::Menu => {
                        let depth = session.menu().map_or(0, |m| m.path.len());
                        if level == depth {
                            vec![Command::MenuActivate(Some(index))]
                        } else if level == 0 {
                            vec![Command::MenuOpenAt(index)]
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                }
            }
            HitTarget::PromptButton(btn) => {
                self.drag = None;
                self.last_press = None;
                let action = match btn {
                    PromptButton::Ok | PromptButton::Yes => Command::PromptSubmit,
                    PromptButton::Cancel | PromptButton::No => Command::PromptCancel,
                };
                vec![Command::PromptFocus(btn), action]
            }
            _ => {
                self.drag = None;
                self.last_press = None;
                Vec::new()
            }
        }
    }

    fn map_pointer_move(
        &mut self,
        col: u16,
        row: u16,
        buttons_down: bool,
        layout: &LayoutMap,
        session: &Session,
    ) -> Vec<Command> {
        if buttons_down && let Some(drag) = self.drag {
            if let HitTarget::GridCell(addr) = layout.hit(col, row) {
                return match drag {
                    DragKind::Grid => vec![Command::NavExtendSelection(addr)],
                    DragKind::Point { anchor } | DragKind::Prompt { anchor } => {
                        let a = if anchor == addr { None } else { Some(anchor) };
                        vec![Command::PointSetReference {
                            anchor: a,
                            cursor: addr,
                        }]
                    }
                };
            }
            if let Some(dir) = layout.edge_for(col, row) {
                let (dcol, drow) = match dir {
                    EdgeDir::Up => (0, -1),
                    EdgeDir::Down => (0, 1),
                    EdgeDir::Left => (-1, 0),
                    EdgeDir::Right => (1, 0),
                };
                let mut cmds = vec![Command::ViewScroll { dcol, drow }];
                let base = layout
                    .nearest_grid_cell(col, row)
                    .unwrap_or_else(|| session.active());
                let next = offset_clamped(base, dcol, drow);
                match drag {
                    DragKind::Grid => cmds.push(Command::NavExtendSelection(next)),
                    DragKind::Point { anchor } | DragKind::Prompt { anchor } => {
                        let a = if anchor == next { None } else { Some(anchor) };
                        cmds.push(Command::PointSetReference {
                            anchor: a,
                            cursor: next,
                        });
                    }
                }
                return cmds;
            }
            return Vec::new();
        }

        let target = layout.hit(col, row);
        if self.hover == Some(target) {
            return Vec::new();
        }
        let was_menu = matches!(self.hover, Some(HitTarget::MenuItem { .. }));
        self.hover = Some(target);
        match target {
            HitTarget::MenuItem { index, .. } => vec![Command::MenuHover(Some(index))],
            HitTarget::PromptButton(btn) => vec![Command::PromptFocus(btn)],
            _ if was_menu => vec![Command::MenuHover(None)],
            _ => Vec::new(),
        }
    }
}

fn map_wheel(dx: i32, dy: i32, mods: Mods) -> Vec<Command> {
    let (sc, sr) = if mods.shift && dx == 0 {
        (i64::from(dy) * WHEEL_ROWS_PER_NOTCH, 0)
    } else {
        (
            i64::from(dx) * WHEEL_ROWS_PER_NOTCH,
            i64::from(dy) * WHEEL_ROWS_PER_NOTCH,
        )
    };
    if sc == 0 && sr == 0 {
        Vec::new()
    } else {
        vec![Command::ViewScroll { dcol: sc, drow: sr }]
    }
}

fn plain(mods: Mods) -> bool {
    !(mods.ctrl || mods.alt || mods.meta)
}

fn offset_clamped(addr: CellAddr, dcol: i64, drow: i64) -> CellAddr {
    let col = i64::from(addr.col())
        .saturating_add(dcol)
        .clamp(0, i64::from(MAX_COL));
    let row = i64::from(addr.row())
        .saturating_add(drow)
        .clamp(0, i64::from(MAX_ROW));
    CellAddr::new(
        u32::try_from(col).unwrap_or(0),
        u32::try_from(row).unwrap_or(0),
    )
    .unwrap_or(addr)
}

fn map_key(key: Key, mods: Mods, session: &Session) -> Vec<Command> {
    let step = |dcol: i64, drow: i64| -> Vec<Command> {
        if mods.shift || session.extend_mode() {
            vec![Command::NavExtendSelection(offset_clamped(
                session.active(),
                dcol,
                drow,
            ))]
        } else {
            vec![Command::NavMoveRelative { dcol, drow }]
        }
    };
    let commit = |dcol: i64, drow: i64| vec![Command::CellCommitEdit(Then::Move { dcol, drow })];
    let caret = |k: CaretKey| vec![Command::CellEditCaret(k)];
    match session.mode() {
        Mode::Ready => match key {
            Key::Char('/') if plain(mods) => vec![Command::MenuOpen],
            Key::Char('z' | 'Z') if mods.ctrl && !mods.alt && !mods.meta => {
                vec![Command::EditUndo]
            }
            Key::Char('y' | 'Y') if mods.ctrl && !mods.alt && !mods.meta => {
                vec![Command::EditRedo]
            }
            Key::Char(c) if plain(mods) => vec![Command::CellBeginEntry(c)],
            Key::Left => step(-1, 0),
            Key::Right => step(1, 0),
            Key::Up => step(0, -1),
            Key::Down => step(0, 1),
            Key::Tab if !mods.shift => vec![Command::NavMoveRelative { dcol: 1, drow: 0 }],
            Key::BackTab | Key::Tab => vec![Command::NavMoveRelative { dcol: -1, drow: 0 }],
            Key::Enter if mods.shift => vec![Command::NavMoveRelative { dcol: 0, drow: -1 }],
            Key::Enter => vec![Command::NavMoveRelative { dcol: 0, drow: 1 }],
            Key::PageUp => vec![Command::NavPageScroll {
                horizontal: mods.alt,
                dir: -1,
            }],
            Key::PageDown => vec![Command::NavPageScroll {
                horizontal: mods.alt,
                dir: 1,
            }],
            Key::Home if mods.ctrl => vec![Command::NavJumpHome(HomeKind::A1)],
            Key::Home => vec![Command::NavJumpHome(HomeKind::RowStart)],
            Key::F(2) => vec![Command::CellBeginEdit],
            Key::F(8) if plain(mods) => vec![Command::NavToggleExtendMode],
            Key::Esc if session.extend_mode() => vec![Command::NavToggleExtendMode],
            Key::Delete if plain(mods) => vec![Command::RangeErase],
            _ => Vec::new(),
        },
        Mode::Value | Mode::Label => {
            if session.is_point_eligible() {
                let point_arrow = |dcol: i64, drow: i64| {
                    vec![Command::PointSetReference {
                        anchor: None,
                        cursor: offset_clamped(session.active(), dcol, drow),
                    }]
                };
                match key {
                    Key::Left => return point_arrow(-1, 0),
                    Key::Right => return point_arrow(1, 0),
                    Key::Up => return point_arrow(0, -1),
                    Key::Down => return point_arrow(0, 1),
                    _ => {}
                }
            }
            match key {
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
            }
        }
        Mode::Edit => {
            if session.is_point_eligible() {
                let point_arrow = |dcol: i64, drow: i64| {
                    vec![Command::PointSetReference {
                        anchor: None,
                        cursor: offset_clamped(session.active(), dcol, drow),
                    }]
                };
                match key {
                    Key::Up => return point_arrow(0, -1),
                    Key::Down => return point_arrow(0, 1),
                    Key::Left => return point_arrow(-1, 0),
                    Key::Right => return point_arrow(1, 0),
                    _ => {}
                }
            }
            match key {
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
            }
        }
        Mode::Point => {
            let cur = session
                .edit()
                .and_then(|e| e.point.as_ref())
                .map_or_else(|| session.active(), |p| p.cursor);
            let existing_anchor = session
                .edit()
                .and_then(|e| e.point.as_ref())
                .and_then(|p| p.anchor);
            let move_point = |dcol: i64, drow: i64| -> Vec<Command> {
                let next = offset_clamped(cur, dcol, drow);
                let anchor = if mods.shift || session.extend_mode() {
                    existing_anchor.or(Some(cur))
                } else {
                    None
                };
                vec![Command::PointSetReference {
                    anchor,
                    cursor: next,
                }]
            };
            match key {
                Key::Left => move_point(-1, 0),
                Key::Right => move_point(1, 0),
                Key::Up => move_point(0, -1),
                Key::Down => move_point(0, 1),
                Key::F(8) if plain(mods) => vec![Command::NavToggleExtendMode],
                Key::Char(c) if plain(mods) => vec![Command::CellEditInput(c)],
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
            }
        }
        Mode::Menu => match key {
            Key::Left => vec![Command::MenuMove(-1)],
            Key::Right => vec![Command::MenuMove(1)],
            Key::Enter => vec![Command::MenuActivate(None)],
            Key::Esc => vec![Command::MenuBack],
            Key::Char(c) if plain(mods) && c.is_ascii_alphabetic() => {
                vec![Command::MenuTrigger(c)]
            }
            _ => Vec::new(),
        },
        Mode::Dialog => map_dialog_key(key, mods, session),
    }
}

fn map_dialog_key(key: Key, mods: Mods, session: &Session) -> Vec<Command> {
    let Some(prompt) = session.prompt() else {
        return Vec::new();
    };
    if let Prompt::ConfirmNew { focus } = prompt {
        return match key {
            Key::Enter => vec![Command::PromptSubmit],
            Key::Esc => vec![Command::PromptCancel],
            Key::Left | Key::Right | Key::Tab | Key::BackTab => {
                let next = if *focus == PromptButton::Yes {
                    PromptButton::No
                } else {
                    PromptButton::Yes
                };
                vec![Command::PromptFocus(next)]
            }
            Key::Char(c) if plain(mods) => vec![Command::PromptInput(c)],
            _ => Vec::new(),
        };
    }

    let arrow_point = |dcol: i64, drow: i64| -> Vec<Command> {
        let cur = prompt.pointing_cursor().unwrap_or_else(|| session.active());
        let next = offset_clamped(cur, dcol, drow);
        let anchor = if mods.shift {
            prompt.pointing_anchor().or(Some(cur))
        } else {
            None
        };
        vec![Command::PointSetReference {
            anchor,
            cursor: next,
        }]
    };

    match key {
        Key::Enter => vec![Command::PromptSubmit],
        Key::Esc => vec![Command::PromptCancel],
        Key::Tab | Key::BackTab => {
            let next = match prompt.focus() {
                Some(PromptButton::Ok) => PromptButton::Cancel,
                _ => PromptButton::Ok,
            };
            vec![Command::PromptFocus(next)]
        }
        Key::Up => arrow_point(0, -1),
        Key::Down => arrow_point(0, 1),
        Key::Left if prompt.is_pristine() || prompt.is_pointing() || mods.shift => {
            arrow_point(-1, 0)
        }
        Key::Right if prompt.is_pristine() || prompt.is_pointing() || mods.shift => {
            arrow_point(1, 0)
        }
        Key::Left => vec![Command::PromptCaret(CaretKey::Left)],
        Key::Right => vec![Command::PromptCaret(CaretKey::Right)],
        Key::Home => vec![Command::PromptCaret(CaretKey::Home)],
        Key::End => vec![Command::PromptCaret(CaretKey::End)],
        Key::Backspace => vec![Command::PromptCaret(CaretKey::Backspace)],
        Key::Delete => vec![Command::PromptCaret(CaretKey::Delete)],
        Key::Char(c) if plain(mods) => vec![Command::PromptInput(c)],
        _ => Vec::new(),
    }
}
