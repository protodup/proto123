//! Entry and editing handlers: READY → VALUE/LABEL/EDIT/POINT, buffer editing, commit, cancel
//! (FR-004, FR-015, FR-023; data-model.md "Mode transitions").

use crate::command::{CaretKey, Then};
use crate::entry::classify_entry;
use crate::session::{EditState, Mode, Session, Status, StatusKind};
use crate::strings::StringKey;
use proto123_coord::CellAddr;
use proto123_ops::Op;

/// Byte offset of a character index (clamped to the end).
fn byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map_or(s.len(), |(i, _)| i)
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

impl Session {
    fn is_editing(&self) -> bool {
        matches!(
            self.mode(),
            Mode::Value | Mode::Label | Mode::Edit | Mode::Point
        )
    }

    /// First printable key in READY: digits and `+ - . ( @ =` start a VALUE entry;
    /// anything else, including `' " ^`, starts a LABEL entry (FR-017).
    pub(crate) fn begin_entry(&mut self, c: char) {
        if self.mode() != Mode::Ready {
            return;
        }
        let mode = if c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | '(' | '@' | '=') {
            Mode::Value
        } else {
            Mode::Label
        };
        self.set_mode(mode);
        self.set_edit(Some(EditState {
            buffer: c.to_string(),
            caret: 1,
            origin_mode: mode,
            point: None,
        }));
    }

    /// `F2`, double-click, or formula-bar click: edit existing content or reposition caret.
    pub(crate) fn begin_edit_at(&mut self, addr: Option<CellAddr>, caret: Option<usize>) {
        if self.is_editing() && addr.is_none() {
            if let Some(target_caret) = caret
                && let Some(mut edit) = self.edit().cloned()
            {
                if self.mode() == Mode::Point {
                    self.set_mode(edit.origin_mode);
                    self.set_extend_mode(false);
                }
                edit.caret = target_caret.min(char_len(&edit.buffer));
                edit.point = None;
                self.set_edit(Some(edit));
            }
            return;
        }
        if self.mode() != Mode::Ready {
            return;
        }
        if let Some(a) = addr {
            self.set_active(a);
        }
        let buffer = self
            .calc()
            .content(self.active())
            .map(|c| c.formula_bar_text())
            .unwrap_or_default();
        let len = char_len(&buffer);
        let pos = caret.map_or(len, |c| c.min(len));
        self.set_edit(Some(EditState {
            buffer,
            caret: pos,
            origin_mode: Mode::Edit,
            point: None,
        }));
        self.set_mode(Mode::Edit);
    }

    pub(crate) fn edit_input(&mut self, c: char) {
        if !self.is_editing() {
            return;
        }
        if self.mode() == Mode::Point {
            self.exit_point_keeping_ref();
        }
        let Some(mut edit) = self.edit().cloned() else {
            return;
        };
        let at = byte_index(&edit.buffer, edit.caret);
        edit.buffer.insert(at, c);
        edit.caret += 1;
        self.set_edit(Some(edit));
    }

    pub(crate) fn edit_caret(&mut self, key: CaretKey) {
        if !self.is_editing() {
            return;
        }
        if self.mode() == Mode::Point {
            self.exit_point_keeping_ref();
        }
        let Some(mut edit) = self.edit().cloned() else {
            return;
        };
        let len = char_len(&edit.buffer);
        match key {
            CaretKey::Left => edit.caret = edit.caret.saturating_sub(1),
            CaretKey::Right => edit.caret = (edit.caret + 1).min(len),
            CaretKey::Home => edit.caret = 0,
            CaretKey::End => edit.caret = len,
            CaretKey::Backspace => {
                if edit.caret > 0 {
                    let start = byte_index(&edit.buffer, edit.caret - 1);
                    let end = byte_index(&edit.buffer, edit.caret);
                    edit.buffer.replace_range(start..end, "");
                    edit.caret -= 1;
                }
            }
            CaretKey::Delete => {
                if edit.caret < len {
                    let start = byte_index(&edit.buffer, edit.caret);
                    let end = byte_index(&edit.buffer, edit.caret + 1);
                    edit.buffer.replace_range(start..end, "");
                }
            }
        }
        self.set_edit(Some(edit));
    }

    /// Commits the buffer. A syntax error keeps the editor open with the caret at the error
    /// (FR-015). Returns true when cell contents changed.
    pub(crate) fn commit_edit(&mut self, then: Then) -> bool {
        if !self.is_editing() {
            return false;
        }
        if self.mode() == Mode::Point {
            self.exit_point_keeping_ref();
        }
        let Some(edit) = self.edit().cloned() else {
            return false;
        };
        let content = match classify_entry(&edit.buffer) {
            Ok(content) => content,
            Err(e) => {
                let caret = e.pos.min(char_len(&edit.buffer));
                self.set_edit(Some(EditState {
                    buffer: edit.buffer,
                    caret,
                    origin_mode: edit.origin_mode,
                    point: None,
                }));
                let hint = e.quote_hint.then_some(StringKey::DiagUseQuoteForText);
                self.set_status(Status {
                    key: StringKey::DiagSyntax,
                    pos: Some(e.pos),
                    hint,
                    kind: StatusKind::Error,
                });
                return false;
            }
        };
        self.set_edit(None);
        self.set_extend_mode(false);
        self.set_mode(Mode::Ready);
        let active = self.active();
        let changed = self.apply_op(Op::SetCell {
            addr: active,
            content,
        });
        match then {
            Then::Move { dcol, drow } => self.move_relative(dcol, drow),
            Then::Select(addr) => self.set_active(addr),
            Then::Stay => {}
        }
        changed
    }

    pub(crate) fn cancel_edit(&mut self) {
        if !self.is_editing() {
            return;
        }
        self.set_edit(None);
        self.set_extend_mode(false);
        self.set_mode(Mode::Ready);
    }
}
