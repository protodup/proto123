//! Command registry (contracts/commands.md). `CommandId` names every state change;
//! `Command` carries its parameters into `Session::dispatch`.

use proto123_coord::CellAddr;
use proto123_theme::{GlyphSet, ThemeId};

/// Stable identifiers, one per registered command. Menu and parity checks use `name`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandId {
    NavMoveRelative,
    NavSelectCell,
    NavExtendSelection,
    NavToggleExtendMode,
    NavPageScroll,
    NavJumpHome,
    ViewScroll,
    SessionResize,
    SessionSetTheme,
    CellBeginEntry,
    CellBeginEdit,
    CellEditInput,
    CellEditCaret,
    CellCommitEdit,
    CellCancelEdit,
    PointSetReference,
    RangeErase,
    RangeCopy,
    RangeMove,
    EditUndo,
    EditRedo,
    FileNewWorkbook,
    ViewCycleTheme,
    MenuOpen,
    MenuOpenAt,
    MenuMove,
    MenuHover,
    MenuActivate,
    MenuTrigger,
    MenuBack,
    MenuClose,
    PromptInput,
    PromptCaret,
    PromptSubmit,
    PromptCancel,
    PromptFocus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    pub id: CommandId,
    pub name: &'static str,
    /// True when the command creates one undo step.
    pub undoable: bool,
}

/// Every registered command. `Session::dispatch` handles every variant.
pub const REGISTRY: &[CommandSpec] = &[
    CommandSpec {
        id: CommandId::NavMoveRelative,
        name: "nav.moveRelative",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::NavSelectCell,
        name: "nav.selectCell",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::NavExtendSelection,
        name: "nav.extendSelection",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::NavToggleExtendMode,
        name: "nav.toggleExtendMode",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::NavPageScroll,
        name: "nav.pageScroll",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::NavJumpHome,
        name: "nav.jumpHome",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::ViewScroll,
        name: "view.scroll",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::SessionResize,
        name: "session.resize",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::SessionSetTheme,
        name: "session.setTheme",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::CellBeginEntry,
        name: "cell.beginEntry",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::CellBeginEdit,
        name: "cell.beginEdit",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::CellEditInput,
        name: "cell.editInput",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::CellEditCaret,
        name: "cell.editCaret",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::CellCommitEdit,
        name: "cell.commitEdit",
        undoable: true,
    },
    CommandSpec {
        id: CommandId::CellCancelEdit,
        name: "cell.cancelEdit",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::PointSetReference,
        name: "point.setReference",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::RangeErase,
        name: "range.erase",
        undoable: true,
    },
    CommandSpec {
        id: CommandId::RangeCopy,
        name: "range.copy",
        undoable: true,
    },
    CommandSpec {
        id: CommandId::RangeMove,
        name: "range.move",
        undoable: true,
    },
    CommandSpec {
        id: CommandId::EditUndo,
        name: "edit.undo",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::EditRedo,
        name: "edit.redo",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::FileNewWorkbook,
        name: "file.newWorkbook",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::ViewCycleTheme,
        name: "view.cycleTheme",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuOpen,
        name: "menu.open",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuOpenAt,
        name: "menu.openAt",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuMove,
        name: "menu.move",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuHover,
        name: "menu.hover",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuActivate,
        name: "menu.activate",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuTrigger,
        name: "menu.trigger",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuBack,
        name: "menu.back",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::MenuClose,
        name: "menu.close",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::PromptInput,
        name: "prompt.input",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::PromptCaret,
        name: "prompt.caret",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::PromptSubmit,
        name: "prompt.submit",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::PromptCancel,
        name: "prompt.cancel",
        undoable: false,
    },
    CommandSpec {
        id: CommandId::PromptFocus,
        name: "prompt.focus",
        undoable: false,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeKind {
    /// `Home`: first column of the current row.
    RowStart,
    /// `Ctrl+Home`: cell A1.
    A1,
}

/// Editing-buffer keys that move the caret or delete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretKey {
    Left,
    Right,
    Home,
    End,
    Backspace,
    Delete,
}

/// Where the active cell goes after a commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Then {
    Move { dcol: i64, drow: i64 },
    Select(CellAddr),
    Stay,
}

/// Interactive prompt buttons in `DIALOG` mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptButton {
    Ok,
    Cancel,
    Yes,
    No,
}

/// A command with its parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    NavMoveRelative {
        dcol: i64,
        drow: i64,
    },
    NavSelectCell(CellAddr),
    NavExtendSelection(CellAddr),
    NavToggleExtendMode,
    /// Moves the active cell by one visible page (`horizontal` = columns, else rows).
    NavPageScroll {
        horizontal: bool,
        dir: i64,
    },
    NavJumpHome(HomeKind),
    ViewScroll {
        dcol: i64,
        drow: i64,
    },
    /// `cols`/`rows` are the visible grid size, computed by the layout (`proto123-tui`).
    SessionResize {
        cols: u32,
        rows: u32,
    },
    SessionResizeWithMetrics {
        cols: u32,
        rows: u32,
        metrics_mismatch: bool,
    },
    SessionSetTheme {
        id: ThemeId,
        glyphs: Option<GlyphSet>,
    },
    CellBeginEntry(char),
    CellBeginEdit,
    CellBeginEditAt {
        addr: Option<CellAddr>,
        caret: Option<usize>,
    },
    CellEditInput(char),
    CellEditCaret(CaretKey),
    CellCommitEdit(Then),
    CellCancelEdit,
    /// Sets or updates the pointed cell/range reference (used in range prompts and `POINT`).
    PointSetReference {
        anchor: Option<CellAddr>,
        cursor: CellAddr,
    },
    RangeErase,
    RangeCopy,
    RangeMove,
    EditUndo,
    EditRedo,
    FileNewWorkbook,
    ViewCycleTheme,
    MenuOpen,
    MenuOpenAt(usize),
    MenuMove(i8),
    MenuHover(Option<usize>),
    MenuActivate(Option<usize>),
    MenuTrigger(char),
    MenuBack,
    MenuClose,
    PromptInput(char),
    PromptCaret(CaretKey),
    PromptSubmit,
    PromptCancel,
    PromptFocus(PromptButton),
}

impl Command {
    pub fn id(&self) -> CommandId {
        match self {
            Self::NavMoveRelative { .. } => CommandId::NavMoveRelative,
            Self::NavSelectCell(_) => CommandId::NavSelectCell,
            Self::NavExtendSelection(_) => CommandId::NavExtendSelection,
            Self::NavToggleExtendMode => CommandId::NavToggleExtendMode,
            Self::NavPageScroll { .. } => CommandId::NavPageScroll,
            Self::NavJumpHome(_) => CommandId::NavJumpHome,
            Self::ViewScroll { .. } => CommandId::ViewScroll,
            Self::SessionResize { .. } | Self::SessionResizeWithMetrics { .. } => {
                CommandId::SessionResize
            }
            Self::SessionSetTheme { .. } => CommandId::SessionSetTheme,
            Self::CellBeginEntry(_) => CommandId::CellBeginEntry,
            Self::CellBeginEdit | Self::CellBeginEditAt { .. } => CommandId::CellBeginEdit,
            Self::CellEditInput(_) => CommandId::CellEditInput,
            Self::CellEditCaret(_) => CommandId::CellEditCaret,
            Self::CellCommitEdit(_) => CommandId::CellCommitEdit,
            Self::CellCancelEdit => CommandId::CellCancelEdit,
            Self::PointSetReference { .. } => CommandId::PointSetReference,
            Self::RangeErase => CommandId::RangeErase,
            Self::RangeCopy => CommandId::RangeCopy,
            Self::RangeMove => CommandId::RangeMove,
            Self::EditUndo => CommandId::EditUndo,
            Self::EditRedo => CommandId::EditRedo,
            Self::FileNewWorkbook => CommandId::FileNewWorkbook,
            Self::ViewCycleTheme => CommandId::ViewCycleTheme,
            Self::MenuOpen => CommandId::MenuOpen,
            Self::MenuOpenAt(_) => CommandId::MenuOpenAt,
            Self::MenuMove(_) => CommandId::MenuMove,
            Self::MenuHover(_) => CommandId::MenuHover,
            Self::MenuActivate(_) => CommandId::MenuActivate,
            Self::MenuTrigger(_) => CommandId::MenuTrigger,
            Self::MenuBack => CommandId::MenuBack,
            Self::MenuClose => CommandId::MenuClose,
            Self::PromptInput(_) => CommandId::PromptInput,
            Self::PromptCaret(_) => CommandId::PromptCaret,
            Self::PromptSubmit => CommandId::PromptSubmit,
            Self::PromptCancel => CommandId::PromptCancel,
            Self::PromptFocus(_) => CommandId::PromptFocus,
        }
    }
}
