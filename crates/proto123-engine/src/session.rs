//! Session state and the exhaustive command dispatch.

use crate::announce::build_announcement;
use crate::command::Command;
use crate::menu::MenuState;
use crate::point::PointState;
use crate::prompt::Prompt;
use crate::strings::{StringKey, text};
use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, CellRange};
use proto123_ops::UndoStack;
use proto123_theme::{GlyphSet, Theme, ThemeId, for_id};

/// Interaction modes (FR-017). Exactly one is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Ready,
    Value,
    Label,
    Edit,
    Point,
    Menu,
    Dialog,
}

impl Mode {
    pub fn badge(self) -> StringKey {
        match self {
            Self::Ready => StringKey::BadgeReady,
            Self::Value => StringKey::BadgeValue,
            Self::Label => StringKey::BadgeLabel,
            Self::Edit => StringKey::BadgeEdit,
            Self::Point => StringKey::BadgePoint,
            Self::Menu => StringKey::BadgeMenu,
            Self::Dialog => StringKey::BadgeDialog,
        }
    }
}

/// The buffer being typed into, with a character-index caret (FR-023).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditState {
    pub buffer: String,
    pub caret: usize,
    pub origin_mode: Mode,
    pub point: Option<PointState>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub redraw: Redraw,
    pub announce: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Session {
    calc: CalcSheet,
    undo: UndoStack,
    mode: Mode,
    active: CellAddr,
    anchor: Option<CellAddr>,
    extend_mode: bool,
    viewport: Viewport,
    /// Visible grid size in cells `(cols, rows)`, set by the layout through `session.resize`.
    view: (u32, u32),
    edit: Option<EditState>,
    menu: Option<MenuState>,
    root_hover: Option<usize>,
    prompt: Option<Prompt>,
    status: Option<Status>,
    theme: Theme,
    announce: Option<String>,
}

impl Session {
    pub fn new(theme: Theme) -> Self {
        Self {
            calc: CalcSheet::new(),
            undo: UndoStack::new(),
            mode: Mode::Ready,
            active: CellAddr::ORIGIN,
            anchor: None,
            extend_mode: false,
            viewport: Viewport { left: 0, top: 0 },
            view: (8, 20),
            edit: None,
            menu: None,
            root_hover: None,
            prompt: None,
            status: None,
            theme,
            announce: None,
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

    pub fn anchor(&self) -> Option<CellAddr> {
        self.anchor
    }

    /// Current selection rectangle (single active cell when no anchor is set).
    pub fn selection(&self) -> CellRange {
        match self.anchor {
            Some(a) => CellRange::new(a, self.active),
            None => CellRange::new(self.active, self.active),
        }
    }

    pub fn extend_mode(&self) -> bool {
        self.extend_mode
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

    pub fn menu(&self) -> Option<&MenuState> {
        self.menu.as_ref()
    }

    pub fn root_hover(&self) -> Option<usize> {
        self.root_hover
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        self.prompt.as_ref()
    }

    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    /// Applies one command. Never panics; invalid input is reported through `status`.
    pub fn dispatch(&mut self, command: Command) -> Outcome {
        let before = (
            self.mode,
            self.active,
            self.anchor,
            self.extend_mode,
            self.viewport,
            self.view,
            self.edit.clone(),
            self.menu.clone(),
            self.root_hover,
            self.prompt.clone(),
            self.status,
        );
        let before_announce = (
            self.mode,
            self.active,
            self.menu
                .as_ref()
                .map(|m| (m.path.clone(), m.active_index())),
            self.root_hover,
        );
        let prev_theme = self.theme;
        self.status = None;
        let mut cells_changed = false;
        let mut full = false;
        match command {
            Command::NavMoveRelative { dcol, drow } => self.move_relative(dcol, drow),
            Command::NavSelectCell(addr) => self.set_active(addr),
            Command::NavExtendSelection(addr) => self.extend_selection(addr),
            Command::NavToggleExtendMode => self.toggle_extend_mode(),
            Command::NavPageScroll { horizontal, dir } => self.page_scroll(horizontal, dir),
            Command::NavJumpHome(kind) => self.jump_home(kind),
            Command::ViewScroll { dcol, drow } => self.scroll_view(dcol, drow),
            Command::SessionResize { cols, rows } => {
                self.view = (cols.max(1), rows.max(1));
                self.follow_active();
                full = true;
            }
            Command::SessionResizeWithMetrics {
                cols,
                rows,
                metrics_mismatch,
            } => {
                self.view = (cols.max(1), rows.max(1));
                self.follow_active();
                if metrics_mismatch {
                    self.set_status(Status {
                        key: StringKey::DiagFontMetricsMismatch,
                        pos: None,
                        hint: None,
                        kind: StatusKind::Error,
                    });
                }
                full = true;
            }
            Command::SessionSetTheme { id, glyphs } => {
                self.apply_theme(id, glyphs);
                full = true;
            }
            Command::CellBeginEntry(c) => self.begin_entry(c),
            Command::CellBeginEdit => self.begin_edit_at(None, None),
            Command::CellBeginEditAt { addr, caret } => self.begin_edit_at(addr, caret),
            Command::CellEditInput(c) => self.edit_input(c),
            Command::CellEditCaret(key) => self.edit_caret(key),
            Command::CellCommitEdit(then) => cells_changed = self.commit_edit(then),
            Command::CellCancelEdit => self.cancel_edit(),
            Command::PointSetReference { anchor, cursor } => {
                self.point_set_reference(anchor, cursor)
            }
            Command::RangeErase => cells_changed = self.erase_selection(),
            Command::RangeCopy => self.open_copy_prompt(),
            Command::RangeMove => self.open_move_prompt(),
            Command::EditUndo => cells_changed = self.undo(),
            Command::EditRedo => cells_changed = self.redo(),
            Command::FileNewWorkbook => cells_changed = self.file_new_workbook(),
            Command::ViewCycleTheme => {
                self.cycle_theme();
                full = true;
            }
            Command::MenuOpen => self.menu_open(),
            Command::MenuOpenAt(idx) => cells_changed = self.menu_open_at(idx),
            Command::MenuMove(delta) => self.menu_move(delta),
            Command::MenuHover(idx) => self.menu_hover(idx),
            Command::MenuActivate(idx) => cells_changed = self.menu_activate(idx),
            Command::MenuTrigger(ch) => cells_changed = self.menu_trigger(ch),
            Command::MenuBack => self.menu_back(),
            Command::MenuClose => self.menu_close(),
            Command::PromptInput(c) => cells_changed = self.prompt_input(c),
            Command::PromptCaret(key) => self.prompt_caret(key),
            Command::PromptSubmit => cells_changed = self.prompt_submit(),
            Command::PromptCancel => self.prompt_cancel(),
            Command::PromptFocus(btn) => self.prompt_focus(btn),
        }
        if self.theme != prev_theme {
            full = true;
        }
        let after = (
            self.mode,
            self.active,
            self.anchor,
            self.extend_mode,
            self.viewport,
            self.view,
            self.edit.clone(),
            self.menu.clone(),
            self.root_hover,
            self.prompt.clone(),
            self.status,
        );
        let after_announce = (
            self.mode,
            self.active,
            self.menu
                .as_ref()
                .map(|m| (m.path.clone(), m.active_index())),
            self.root_hover,
        );
        if before_announce != after_announce || self.status.is_some() {
            self.announce = Some(build_announcement(self));
        }
        let redraw = if full {
            Redraw::Full
        } else if cells_changed || before != after {
            Redraw::Regions
        } else {
            Redraw::None
        };
        Outcome {
            redraw,
            announce: self.announce.take(),
        }
    }

    pub(crate) fn cycle_theme(&mut self) {
        let next = self.theme.id.next();
        self.theme = for_id(next);
    }

    pub(crate) fn apply_theme(&mut self, id: ThemeId, glyphs: Option<GlyphSet>) {
        let mut next = for_id(id);
        if let Some(g) = glyphs {
            next.glyphs = g;
        }
        self.theme = next;
    }

    pub(crate) fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

    pub(crate) fn set_active_raw(&mut self, addr: CellAddr) {
        self.active = addr;
    }

    pub(crate) fn set_anchor(&mut self, anchor: Option<CellAddr>) {
        self.anchor = anchor;
    }

    pub(crate) fn set_extend_mode(&mut self, extend: bool) {
        self.extend_mode = extend;
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

    pub(crate) fn set_menu(&mut self, menu: Option<MenuState>) {
        self.menu = menu;
    }

    pub(crate) fn set_root_hover(&mut self, hover: Option<usize>) {
        self.root_hover = hover;
    }

    pub(crate) fn set_prompt(&mut self, prompt: Option<Prompt>) {
        self.prompt = prompt;
    }

    pub(crate) fn calc_mut(&mut self) -> &mut CalcSheet {
        &mut self.calc
    }

    pub(crate) fn undo_mut(&mut self) -> &mut UndoStack {
        &mut self.undo
    }
}
