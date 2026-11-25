//! Session state and the exhaustive command dispatch.

use crate::command::Command;
use crate::strings::{StringKey, text};
use proto123_calc::CalcSheet;
use proto123_coord::CellAddr;
use proto123_theme::Theme;

/// Interaction modes (FR-017). Exactly one is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Ready,
    Value,
    Label,
    Edit,
}

impl Mode {
    pub fn badge(self) -> StringKey {
        match self {
            Self::Ready => StringKey::BadgeReady,
            Self::Value => StringKey::BadgeValue,
            Self::Label => StringKey::BadgeLabel,
            Self::Edit => StringKey::BadgeEdit,
        }
    }
}

/// The buffer being typed into, with a character-index caret (FR-023).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditState {
    pub buffer: String,
    pub caret: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Error,
}

/// A status-bar message. `pos` fills the `{pos}` placeholder; `hint` is a second line of advice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub key: StringKey,
    pub pos: Option<usize>,
    pub hint: Option<StringKey>,
    pub kind: StatusKind,
}

impl Status {
    pub fn message(&self) -> String {
        let mut out = match self.pos {
            Some(p) => text(self.key).replace("{pos}", &(p + 1).to_string()),
            None => text(self.key).to_owned(),
        };
        if let Some(hint) = self.hint {
            out.push(' ');
            out.push_str(text(hint));
        }
        out
    }
}

/// Scroll origin: the first visible column and row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub left: u32,
    pub top: u32,
}

/// What the front end must repaint after a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Redraw {
    None,
    /// Only the cells, headers, control line, and status bar that changed.
    Regions,
    /// Everything (resize, theme change).
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub redraw: Redraw,
}

#[derive(Clone, Debug)]
pub struct Session {
    calc: CalcSheet,
    mode: Mode,
    active: CellAddr,
    viewport: Viewport,
    /// Visible grid size in cells `(cols, rows)`, set by the layout through `session.resize`.
    view: (u32, u32),
    edit: Option<EditState>,
    status: Option<Status>,
    theme: Theme,
}

impl Session {
    pub fn new(theme: Theme) -> Self {
        Self {
            calc: CalcSheet::new(),
            mode: Mode::Ready,
            active: CellAddr::ORIGIN,
            viewport: Viewport { left: 0, top: 0 },
            view: (8, 20),
            edit: None,
            status: None,
            theme,
        }
    }

    pub fn calc(&self) -> &CalcSheet {
        &self.calc
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn active(&self) -> CellAddr {
        self.active
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn view_size(&self) -> (u32, u32) {
        self.view
    }

    pub fn edit(&self) -> Option<&EditState> {
        self.edit.as_ref()
    }

    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Applies one command. Never panics; invalid input is reported through `status`.
    pub fn dispatch(&mut self, command: Command) -> Outcome {
        let before = (self.mode, self.active, self.viewport, self.view, self.edit.clone(), self.status);
        self.status = None;
        let mut cells_changed = false;
        let mut full = false;
        match command {
            Command::NavMoveRelative { dcol, drow } => self.move_relative(dcol, drow),
            Command::NavSelectCell(addr) => self.set_active(addr),
            Command::NavPageScroll { horizontal, dir } => self.page_scroll(horizontal, dir),
            Command::NavJumpHome(kind) => self.jump_home(kind),
            Command::ViewScroll { dcol, drow } => self.scroll_view(dcol, drow),
            Command::SessionResize { cols, rows } => {
                self.view = (cols.max(1), rows.max(1));
                self.follow_active();
                full = true;
            }
            Command::CellBeginEntry(c) => self.begin_entry(c),
            Command::CellBeginEdit => self.begin_edit(),
            Command::CellEditInput(c) => self.edit_input(c),
            Command::CellEditCaret(key) => self.edit_caret(key),
            Command::CellCommitEdit(then) => cells_changed = self.commit_edit(then),
            Command::CellCancelEdit => self.cancel_edit(),
        }
        let after = (self.mode, self.active, self.viewport, self.view, self.edit.clone(), self.status);
        let redraw = if full {
            Redraw::Full
        } else if cells_changed || before != after {
            Redraw::Regions
        } else {
            Redraw::None
        };
        Outcome { redraw }
    }

    pub(crate) fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

    pub(crate) fn set_active_raw(&mut self, addr: CellAddr) {
        self.active = addr;
    }

    pub(crate) fn set_viewport(&mut self, left: u32, top: u32) {
        self.viewport = Viewport { left, top };
    }

    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub(crate) fn set_edit(&mut self, edit: Option<EditState>) {
        self.edit = edit;
    }

    pub(crate) fn calc_mut(&mut self) -> &mut CalcSheet {
        &mut self.calc
    }
}

