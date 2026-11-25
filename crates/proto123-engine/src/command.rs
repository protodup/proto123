//! Command registry (contracts/commands.md). `CommandId` names every state change;
//! `Command` carries its parameters into `Session::dispatch`.

use proto123_coord::CellAddr;

/// Stable identifiers, one per registered command. Menu and parity checks use `name`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandId {
    NavMoveRelative,
    NavSelectCell,
    NavPageScroll,
    NavJumpHome,
    ViewScroll,
    SessionResize,
    CellBeginEntry,
    CellBeginEdit,
    CellEditInput,
    CellEditCaret,
    CellCommitEdit,
    CellCancelEdit,
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
    CommandSpec { id: CommandId::NavMoveRelative, name: "nav.moveRelative", undoable: false },
    CommandSpec { id: CommandId::NavSelectCell, name: "nav.selectCell", undoable: false },
    CommandSpec { id: CommandId::NavPageScroll, name: "nav.pageScroll", undoable: false },
    CommandSpec { id: CommandId::NavJumpHome, name: "nav.jumpHome", undoable: false },
    CommandSpec { id: CommandId::ViewScroll, name: "view.scroll", undoable: false },
    CommandSpec { id: CommandId::SessionResize, name: "session.resize", undoable: false },
    CommandSpec { id: CommandId::CellBeginEntry, name: "cell.beginEntry", undoable: false },
    CommandSpec { id: CommandId::CellBeginEdit, name: "cell.beginEdit", undoable: false },
    CommandSpec { id: CommandId::CellEditInput, name: "cell.editInput", undoable: false },
    CommandSpec { id: CommandId::CellEditCaret, name: "cell.editCaret", undoable: false },
    CommandSpec { id: CommandId::CellCommitEdit, name: "cell.commitEdit", undoable: true },
    CommandSpec { id: CommandId::CellCancelEdit, name: "cell.cancelEdit", undoable: false },
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

/// A command with its parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    NavMoveRelative { dcol: i64, drow: i64 },
    NavSelectCell(CellAddr),
    /// Moves the active cell by one visible page (`horizontal` = columns, else rows).
    NavPageScroll { horizontal: bool, dir: i64 },
    NavJumpHome(HomeKind),
    ViewScroll { dcol: i64, drow: i64 },
    /// `cols`/`rows` are the visible grid size, computed by the layout (`proto123-tui`).
    SessionResize { cols: u32, rows: u32 },
    CellBeginEntry(char),
    CellBeginEdit,
    CellEditInput(char),
    CellEditCaret(CaretKey),
    CellCommitEdit(Then),
    CellCancelEdit,
}

impl Command {
    pub fn id(&self) -> CommandId {
        match self {
            Self::NavMoveRelative { .. } => CommandId::NavMoveRelative,
            Self::NavSelectCell(_) => CommandId::NavSelectCell,
            Self::NavPageScroll { .. } => CommandId::NavPageScroll,
            Self::NavJumpHome(_) => CommandId::NavJumpHome,
            Self::ViewScroll { .. } => CommandId::ViewScroll,
            Self::SessionResize { .. } => CommandId::SessionResize,
            Self::CellBeginEntry(_) => CommandId::CellBeginEntry,
            Self::CellBeginEdit => CommandId::CellBeginEdit,
            Self::CellEditInput(_) => CommandId::CellEditInput,
            Self::CellEditCaret(_) => CommandId::CellEditCaret,
            Self::CellCommitEdit(_) => CommandId::CellCommitEdit,
            Self::CellCancelEdit => CommandId::CellCancelEdit,
        }
    }
}
