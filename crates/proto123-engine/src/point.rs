//! `POINT` mode and live reference insertion for formula editing (FR-023, FR-029).

use crate::session::{EditState, Mode, Session};
use proto123_coord::{CellAddr, CellRange};

/// Live reference span currently being pointed into the formula edit buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointState {
    pub insert_at: usize,
    pub len: usize,
    pub anchor: Option<CellAddr>,
    pub cursor: CellAddr,
}

fn byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map_or(s.len(), |(i, _)| i)
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// True when `edit` is a formula and the caret immediately follows an operator, `(`, `,`, `;`, or `:`
/// (FR-029, data-model.md "Mode transitions").
pub fn is_point_eligible(edit: &EditState) -> bool {
    let starts_formula = matches!(edit.buffer.chars().next(), Some('=' | '+' | '@'));
    if !starts_formula || edit.caret == 0 {
        return false;
    }
    let Some(prev) = edit.buffer.chars().nth(edit.caret - 1) else {
        return false;
    };
    matches!(
        prev,
        '=' | '+' | '-' | '*' | '/' | '^' | '(' | ',' | ';' | ':'
    )
}

fn format_ref(anchor: Option<CellAddr>, cursor: CellAddr) -> String {
    match anchor {
        Some(a) if a != cursor => CellRange::new(a, cursor).to_string(),
        _ => cursor.to_string(),
    }
}

impl Session {
    pub fn is_point_eligible(&self) -> bool {
        matches!(self.mode(), Mode::Value | Mode::Edit)
            && self.edit().is_some_and(is_point_eligible)
    }

    pub(crate) fn point_set_reference(&mut self, anchor: Option<CellAddr>, cursor: CellAddr) {
        if self.mode() == Mode::Dialog {
            self.point_in_prompt(anchor, cursor);
            return;
        }
        if self.mode() == Mode::Point {
            let Some(mut edit) = self.edit().cloned() else {
                return;
            };
            let Some(mut pt) = edit.point.clone() else {
                return;
            };
            let text = format_ref(anchor, cursor);
            let start = byte_index(&edit.buffer, pt.insert_at);
            let end = byte_index(&edit.buffer, pt.insert_at + pt.len);
            edit.buffer.replace_range(start..end, &text);
            let new_len = char_len(&text);
            pt.len = new_len;
            pt.anchor = anchor;
            pt.cursor = cursor;
            edit.caret = pt.insert_at + new_len;
            edit.point = Some(pt);
            self.set_edit(Some(edit));
            self.follow_addr(cursor);
            return;
        }
        if !self.is_point_eligible() {
            return;
        }
        let Some(mut edit) = self.edit().cloned() else {
            return;
        };
        let text = format_ref(anchor, cursor);
        let insert_at = edit.caret;
        let at_byte = byte_index(&edit.buffer, insert_at);
        edit.buffer.insert_str(at_byte, &text);
        let len = char_len(&text);
        edit.caret = insert_at + len;
        edit.origin_mode = self.mode();
        edit.point = Some(PointState {
            insert_at,
            len,
            anchor,
            cursor,
        });
        self.set_edit(Some(edit));
        self.set_mode(Mode::Point);
        self.follow_addr(cursor);
    }

    /// Leaves `POINT` back to `origin_mode` while keeping the inserted reference in the buffer.
    pub(crate) fn exit_point_keeping_ref(&mut self) {
        if self.mode() != Mode::Point {
            return;
        }
        let Some(mut edit) = self.edit().cloned() else {
            self.set_mode(Mode::Edit);
            return;
        };
        let origin = edit.origin_mode;
        edit.point = None;
        self.set_edit(Some(edit));
        self.set_extend_mode(false);
        self.set_mode(origin);
    }
}
