//! Atomic operation application, range erase, and Undo/Redo (FR-018).

use crate::session::{Session, Status, StatusKind};
use crate::strings::StringKey;
use proto123_ops::Op;

impl Session {
    /// Plans and applies `op`, pushing a non-empty delta onto the undo stack.
    /// Returns true when the visible sheet changed.
    pub(crate) fn apply_op(&mut self, op: Op) -> bool {
        let delta = op.plan(self.calc());
        if delta.is_empty() {
            return false;
        }
        let after = delta.after_changes();
        let report = self.calc_mut().apply(&after);
        self.undo_mut().push(delta);
        !report.changed_display.is_empty() || !report.evaluated.is_empty() || !after.is_empty()
    }

    pub(crate) fn erase_selection(&mut self) -> bool {
        if self.mode() != crate::session::Mode::Ready {
            return false;
        }
        let range = self.selection();
        self.apply_op(Op::Erase { range })
    }

    pub(crate) fn undo(&mut self) -> bool {
        if self.mode() != crate::session::Mode::Ready {
            return false;
        }
        let Some(delta) = self.undo_mut().undo() else {
            self.set_status(Status {
                key: StringKey::StatusNothingToUndo,
                pos: None,
                hint: None,
                kind: StatusKind::Info,
            });
            return false;
        };
        self.calc_mut().apply(&delta.before_changes());
        true
    }

    pub(crate) fn redo(&mut self) -> bool {
        if self.mode() != crate::session::Mode::Ready {
            return false;
        }
        let Some(delta) = self.undo_mut().redo() else {
            self.set_status(Status {
                key: StringKey::StatusNothingToRedo,
                pos: None,
                hint: None,
                kind: StatusKind::Info,
            });
            return false;
        };
        self.calc_mut().apply(&delta.after_changes());
        true
    }
}
