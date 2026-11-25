//! Dialog prompts (`DIALOG` mode): Copy/Move range prompts and File New confirmation.

use crate::command::{CaretKey, PromptButton};
use crate::session::{Mode, Session, Status, StatusKind, Viewport};
use crate::strings::StringKey;
use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, CellRange, MAX_COL, MAX_ROW, RefAddr};
use proto123_ops::Op;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangePromptKind {
    Copy,
    Move,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prompt {
    ConfirmNew {
        focus: PromptButton,
    },
    RangeSource {
        kind: RangePromptKind,
        text: String,
        caret: usize,
        pristine: bool,
        anchor: Option<CellAddr>,
        cursor: Option<CellAddr>,
        focus: Option<PromptButton>,
    },
    RangeDest {
        kind: RangePromptKind,
        src: CellRange,
        text: String,
        caret: usize,
        pristine: bool,
        anchor: Option<CellAddr>,
        cursor: Option<CellAddr>,
        focus: Option<PromptButton>,
    },
}

impl Prompt {
    pub fn label_key(&self) -> StringKey {
        match self {
            Self::ConfirmNew { .. } => StringKey::PromptConfirmNew,
            Self::RangeSource {
                kind: RangePromptKind::Copy,
                ..
            } => StringKey::PromptCopySource,
            Self::RangeDest {
                kind: RangePromptKind::Copy,
                ..
            } => StringKey::PromptCopyDest,
            Self::RangeSource {
                kind: RangePromptKind::Move,
                ..
            } => StringKey::PromptMoveSource,
            Self::RangeDest {
                kind: RangePromptKind::Move,
                ..
            } => StringKey::PromptMoveDest,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::ConfirmNew { .. } => "",
            Self::RangeSource { text, .. } | Self::RangeDest { text, .. } => text,
        }
    }

    pub fn caret(&self) -> usize {
        match self {
            Self::ConfirmNew { .. } => 0,
            Self::RangeSource { caret, .. } | Self::RangeDest { caret, .. } => *caret,
        }
    }

    pub fn is_pristine(&self) -> bool {
        match self {
            Self::ConfirmNew { .. } => false,
            Self::RangeSource { pristine, .. } | Self::RangeDest { pristine, .. } => *pristine,
        }
    }

    pub fn is_pointing(&self) -> bool {
        match self {
            Self::ConfirmNew { .. } => false,
            Self::RangeSource { cursor, .. } | Self::RangeDest { cursor, .. } => cursor.is_some(),
        }
    }

    pub fn pointing_cursor(&self) -> Option<CellAddr> {
        match self {
            Self::ConfirmNew { .. } => None,
            Self::RangeSource { cursor, .. } | Self::RangeDest { cursor, .. } => *cursor,
        }
    }

    pub fn pointing_anchor(&self) -> Option<CellAddr> {
        match self {
            Self::ConfirmNew { .. } => None,
            Self::RangeSource { anchor, .. } | Self::RangeDest { anchor, .. } => *anchor,
        }
    }

    pub fn focus(&self) -> Option<PromptButton> {
        match self {
            Self::ConfirmNew { focus } => Some(*focus),
            Self::RangeSource { focus, .. } | Self::RangeDest { focus, .. } => *focus,
        }
    }
}

fn byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map_or(s.len(), |(i, _)| i)
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn format_range_or_cell(range: CellRange) -> String {
    if range.start() == range.end() {
        range.start().to_string()
    } else {
        range.to_string()
    }
}

fn parse_range_or_cell(text: &str) -> Option<CellRange> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(r) = CellRange::parse(trimmed) {
        return Some(r);
    }
    RefAddr::parse(trimmed).map(|r| CellRange::new(r.addr, r.addr))
}

impl Session {
    pub(crate) fn file_new_workbook(&mut self) -> bool {
        if self.calc().workbook().is_empty() {
            self.reset_workbook();
            return false;
        }
        self.set_prompt(Some(Prompt::ConfirmNew {
            focus: PromptButton::No,
        }));
        self.set_mode(Mode::Dialog);
        false
    }

    fn reset_workbook(&mut self) {
        *self.calc_mut() = CalcSheet::new();
        self.undo_mut().clear();
        self.set_active_raw(CellAddr::ORIGIN);
        self.set_anchor(None);
        self.set_extend_mode(false);
        self.set_viewport(Viewport { left: 0, top: 0 }.left, 0);
        self.set_edit(None);
        self.set_prompt(None);
        self.set_menu(None);
        self.set_mode(Mode::Ready);
    }

    pub(crate) fn open_copy_prompt(&mut self) {
        self.open_range_prompt(RangePromptKind::Copy);
    }

    pub(crate) fn open_move_prompt(&mut self) {
        self.open_range_prompt(RangePromptKind::Move);
    }

    fn open_range_prompt(&mut self, kind: RangePromptKind) {
        let initial = format_range_or_cell(self.selection());
        let caret = char_len(&initial);
        self.set_prompt(Some(Prompt::RangeSource {
            kind,
            text: initial,
            caret,
            pristine: true,
            anchor: None,
            cursor: None,
            focus: None,
        }));
        self.set_mode(Mode::Dialog);
    }

    pub(crate) fn prompt_input(&mut self, c: char) -> bool {
        if self.mode() != Mode::Dialog {
            return false;
        }
        let Some(mut prompt) = self.prompt().cloned() else {
            return false;
        };
        match &mut prompt {
            Prompt::ConfirmNew { .. } => match c {
                'y' | 'Y' => {
                    self.reset_workbook();
                    return true;
                }
                'n' | 'N' => {
                    self.prompt_cancel();
                    return false;
                }
                _ => return false,
            },
            Prompt::RangeSource {
                text,
                caret,
                pristine,
                anchor,
                cursor,
                focus,
                ..
            }
            | Prompt::RangeDest {
                text,
                caret,
                pristine,
                anchor,
                cursor,
                focus,
                ..
            } => {
                *anchor = None;
                *cursor = None;
                *focus = None;
                if *pristine {
                    *text = c.to_string();
                    *caret = 1;
                    *pristine = false;
                } else {
                    let at = byte_index(text, *caret);
                    text.insert(at, c);
                    *caret += 1;
                }
            }
        }
        self.set_prompt(Some(prompt));
        false
    }

    pub(crate) fn prompt_caret(&mut self, key: CaretKey) {
        if self.mode() != Mode::Dialog {
            return;
        }
        let Some(mut prompt) = self.prompt().cloned() else {
            return;
        };
        match &mut prompt {
            Prompt::ConfirmNew { focus } => {
                if matches!(key, CaretKey::Left | CaretKey::Right) {
                    *focus = match *focus {
                        PromptButton::Yes => PromptButton::No,
                        _ => PromptButton::Yes,
                    };
                }
            }
            Prompt::RangeSource {
                text,
                caret,
                pristine,
                anchor,
                cursor,
                focus,
                ..
            }
            | Prompt::RangeDest {
                text,
                caret,
                pristine,
                anchor,
                cursor,
                focus,
                ..
            } => {
                *pristine = false;
                *anchor = None;
                *cursor = None;
                *focus = None;
                let len = char_len(text);
                match key {
                    CaretKey::Left => *caret = caret.saturating_sub(1),
                    CaretKey::Right => *caret = (*caret + 1).min(len),
                    CaretKey::Home => *caret = 0,
                    CaretKey::End => *caret = len,
                    CaretKey::Backspace => {
                        if *caret > 0 {
                            let start = byte_index(text, *caret - 1);
                            let end = byte_index(text, *caret);
                            text.replace_range(start..end, "");
                            *caret -= 1;
                        }
                    }
                    CaretKey::Delete => {
                        if *caret < len {
                            let start = byte_index(text, *caret);
                            let end = byte_index(text, *caret + 1);
                            text.replace_range(start..end, "");
                        }
                    }
                }
            }
        }
        self.set_prompt(Some(prompt));
    }

    pub(crate) fn point_in_prompt(&mut self, new_anchor: Option<CellAddr>, new_cursor: CellAddr) {
        if self.mode() != Mode::Dialog {
            return;
        }
        let Some(mut prompt) = self.prompt().cloned() else {
            return;
        };
        if matches!(prompt, Prompt::ConfirmNew { .. }) {
            return;
        }
        let formatted = match new_anchor {
            Some(a) if a != new_cursor => CellRange::new(a, new_cursor).to_string(),
            _ => new_cursor.to_string(),
        };
        let len = char_len(&formatted);
        match &mut prompt {
            Prompt::ConfirmNew { .. } => {}
            Prompt::RangeSource {
                text,
                caret,
                pristine,
                anchor,
                cursor,
                focus,
                ..
            }
            | Prompt::RangeDest {
                text,
                caret,
                pristine,
                anchor,
                cursor,
                focus,
                ..
            } => {
                *text = formatted;
                *caret = len;
                *pristine = false;
                *anchor = new_anchor;
                *cursor = Some(new_cursor);
                *focus = None;
            }
        }
        self.set_prompt(Some(prompt));
        self.follow_addr(new_cursor);
    }

    pub(crate) fn prompt_focus(&mut self, button: PromptButton) {
        if self.mode() != Mode::Dialog {
            return;
        }
        let Some(mut prompt) = self.prompt().cloned() else {
            return;
        };
        match &mut prompt {
            Prompt::ConfirmNew { focus } => {
                if matches!(button, PromptButton::Yes | PromptButton::No) {
                    *focus = button;
                }
            }
            Prompt::RangeSource { focus, .. } | Prompt::RangeDest { focus, .. } => {
                *focus = Some(button);
            }
        }
        self.set_prompt(Some(prompt));
    }

    pub(crate) fn prompt_submit(&mut self) -> bool {
        if self.mode() != Mode::Dialog {
            return false;
        }
        let Some(prompt) = self.prompt().cloned() else {
            return false;
        };
        if matches!(
            prompt.focus(),
            Some(PromptButton::Cancel | PromptButton::No)
        ) {
            self.prompt_cancel();
            return false;
        }
        match prompt {
            Prompt::ConfirmNew { focus } => {
                if focus == PromptButton::Yes {
                    self.reset_workbook();
                    true
                } else {
                    self.prompt_cancel();
                    false
                }
            }
            Prompt::RangeSource { kind, text, .. } => {
                let Some(src) = parse_range_or_cell(&text) else {
                    self.set_status(Status {
                        key: StringKey::DiagInvalidRange,
                        pos: None,
                        hint: None,
                        kind: StatusKind::Error,
                    });
                    return false;
                };
                let dest_default = self.active().to_string();
                let caret = char_len(&dest_default);
                self.set_prompt(Some(Prompt::RangeDest {
                    kind,
                    src,
                    text: dest_default,
                    caret,
                    pristine: true,
                    anchor: None,
                    cursor: None,
                    focus: None,
                }));
                false
            }
            Prompt::RangeDest {
                kind, src, text, ..
            } => {
                let Some(dest) = parse_range_or_cell(&text) else {
                    self.set_status(Status {
                        key: StringKey::DiagInvalidRange,
                        pos: None,
                        hint: None,
                        kind: StatusKind::Error,
                    });
                    return false;
                };
                if kind == RangePromptKind::Move {
                    let end_c = dest
                        .start()
                        .col()
                        .saturating_add(src.width().saturating_sub(1));
                    let end_r = dest
                        .start()
                        .row()
                        .saturating_add(src.height().saturating_sub(1));
                    if end_c > MAX_COL || end_r > MAX_ROW {
                        self.set_status(Status {
                            key: StringKey::DiagMoveOutOfGrid,
                            pos: None,
                            hint: None,
                            kind: StatusKind::Error,
                        });
                        return false;
                    }
                }
                self.set_prompt(None);
                self.set_mode(Mode::Ready);
                let op = match kind {
                    RangePromptKind::Copy => Op::Copy { src, dest },
                    RangePromptKind::Move => Op::Move { src, dest },
                };
                self.apply_op(op)
            }
        }
    }

    pub(crate) fn prompt_cancel(&mut self) {
        if self.mode() != Mode::Dialog {
            return;
        }
        self.set_prompt(None);
        self.set_mode(Mode::Ready);
    }
}
