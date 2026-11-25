//! Draws one frame of the 80×24 anatomy (docs/interaction-spec.md §1) and records every
//! drawn target in the `LayoutMap` in the same pass. All colors and metrics come from the
//! session's theme tokens.

use crate::layout::{ADDRESS_BOX_WIDTH, FOOTER_ROWS, HEADER_ROWS, meets_minimum};
use crate::layout_map::{HitTarget, LayoutMap};
use crate::style::style;
use proto123_coord::{CellAddr, MAX_COL, MAX_ROW, col_letters};
use proto123_engine::{Session, StringKey, text};
use proto123_model::{Alignment, CellContent};
use proto123_value::Value;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use unicode_width::UnicodeWidthStr;

/// Writes `s` at `(x, y)`, clipped to `area`.
fn put(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, st: Style) {
    if y >= area.y.saturating_add(area.height) || x >= area.x.saturating_add(area.width) {
        return;
    }
    buf.set_stringn(x, y, s, usize::from(area.right().saturating_sub(x)), st);
}

/// Pads `s` to `width` characters, aligned left, right, or centered.
fn align_in(s: &str, width: usize, align: Alignment) -> String {
    let len = UnicodeWidthStr::width(s);
    if len >= width {
        return s.chars().take(width).collect();
    }
    let gap = width - len;
    match align {
        Alignment::Left => format!("{s}{}", " ".repeat(gap)),
        Alignment::Right => format!("{}{s}", " ".repeat(gap)),
        Alignment::Center => {
            let left = gap / 2;
            format!("{}{s}{}", " ".repeat(left), " ".repeat(gap - left))
        }
    }
}

/// Text shown in the formula bar (or the live edit buffer) for the control line.
fn control_text(session: &Session) -> (String, Option<usize>) {
    match session.edit() {
        Some(edit) => (edit.buffer.clone(), Some(edit.caret)),
        None => (
            session
                .calc()
                .content(session.active())
                .map(|c| c.formula_bar_text())
                .unwrap_or_default(),
            None,
        ),
    }
}

/// Renders `session` into `area` and returns the targets it drew.
pub fn render(session: &Session, area: Rect, buf: &mut Buffer) -> LayoutMap {
    let theme = session.theme();
    let tokens = theme.tokens;
    let metrics = theme.metrics;
    let mut map = LayoutMap::default();

    buf.set_style(area, style(tokens.grid));
    if !meets_minimum(area.width, area.height, &metrics) {
        let msg = text(StringKey::EnlargeWindow)
            .replace("{cols}", &metrics.min_cols.to_string())
            .replace("{rows}", &metrics.min_rows.to_string());
        put(buf, area, area.x, area.y, &msg, style(tokens.error));
        return map;
    }

    let active = session.active();
    let vp = session.viewport();
    let row_w = metrics.row_header_width;
    let col_w = metrics.col_width;
    let grid_top = area.y + HEADER_ROWS;
    let grid_rows = area.height.saturating_sub(HEADER_ROWS + FOOTER_ROWS);
    let grid_cols = area.width.saturating_sub(row_w) / col_w.max(1);

    // Control line: address box, formula-bar text, mode badge.
    let control = Rect::new(area.x, area.y, area.width, 1);
    buf.set_style(control, style(tokens.control_line));
    let address = format!("[{active}]");
    put(buf, area, area.x, area.y, &format!("{address:<w$}", w = usize::from(ADDRESS_BOX_WIDTH)), style(tokens.header));
    map.push(Rect::new(area.x, area.y, ADDRESS_BOX_WIDTH, 1), HitTarget::AddressBox);

    let badge = text(session.mode().badge());
    let badge_w = u16::try_from(badge.len()).unwrap_or(0);
    let badge_x = area.right().saturating_sub(badge_w + 1);
    put(buf, area, badge_x, area.y, badge, style(tokens.badge));

    let (content, caret) = control_text(session);
    let content_x = area.x + ADDRESS_BOX_WIDTH + 1;
    let content_w = badge_x.saturating_sub(content_x + 1);
    let content_rect = Rect::new(content_x, area.y, content_w, 1);
    put(buf, area, content_x, area.y, &content, style(tokens.control_line));
    map.push(content_rect, HitTarget::FormulaBar { char_index: caret.unwrap_or(0) });
    if let Some(caret) = caret {
        let caret_x = content_x.saturating_add(u16::try_from(caret).unwrap_or(u16::MAX));
        if let Some(cell) = buf.cell_mut((caret_x, area.y)) {
            cell.set_style(Style::default().add_modifier(Modifier::REVERSED));
        }
    }

    // Root-bar line: placeholder until the `/` menu lands.
    let root = Rect::new(area.x, area.y + 1, area.width, 1);
    buf.set_style(root, style(tokens.menu));

    // Column headers.
    let header_y = area.y + 2;
    for i in 0..grid_cols {
        let col = vp.left.saturating_add(u32::from(i));
        if col > MAX_COL {
            break;
        }
        let x = area.x + row_w + i * col_w;
        let letters = col_letters(col);
        let label = if col == active.col() { format!("[{letters}]") } else { letters };
        let st = if col == active.col() { style(tokens.active_cell) } else { style(tokens.header) };
        put(buf, area, x, header_y, &align_in(&label, usize::from(col_w), Alignment::Center), st);
        map.push(Rect::new(x, header_y, col_w, 1), HitTarget::ColHeader(col));
    }

    // Grid rows with row headers and cells.
    for j in 0..grid_rows {
        let row = vp.top.saturating_add(u32::from(j));
        if row > MAX_ROW {
            break;
        }
        let y = grid_top + j;
        let number = format!("{:>w$}", row + 1, w = usize::from(row_w) - 2);
        let is_active_row = row == active.row();
        let header = if is_active_row { format!("[{number}]") } else { format!(" {number} ") };
        let header_st = if is_active_row { style(tokens.active_cell) } else { style(tokens.header) };
        put(buf, area, area.x, y, &header, header_st);
        map.push(Rect::new(area.x, y, row_w, 1), HitTarget::RowHeader(row));

        let mut i: u16 = 0;
        while i < grid_cols {
            let col = vp.left.saturating_add(u32::from(i));
            let Some(addr) = CellAddr::new(col, row) else {
                break;
            };
            let x = area.x + row_w + i * col_w;
            let value = session.calc().value(addr);
            let is_active = addr == active;
            let cell_st = if is_active { style(tokens.active_cell) } else { style(tokens.grid) };
            let (text_out, align, span) = cell_text(session, addr, &value, col_w, i, grid_cols);
            put(buf, area, x, y, &align_in(&text_out, usize::from(span), align), cell_st);
            map.push(Rect::new(x, y, col_w, 1), HitTarget::GridCell(addr));
            // Spilled cells: skip the ones the text now covers, still recording them.
            let covered = span.div_ceil(col_w).max(1);
            for k in 1..covered {
                let skipped_col = col.saturating_add(u32::from(k));
                if let Some(skipped) = CellAddr::new(skipped_col, row) {
                    map.push(
                        Rect::new(x + k * col_w, y, col_w, 1),
                        HitTarget::GridCell(skipped),
                    );
                }
            }
            i += covered;
        }
    }

    // Status bar.
    let status_y = area.y + area.height - 1;
    let status_rect = Rect::new(area.x, status_y, area.width, 1);
    buf.set_style(status_rect, style(tokens.status));
    if let Some(status) = session.status() {
        put(buf, area, area.x, status_y, &status.message(), style(tokens.status));
    }
    map
}

/// The text a cell shows, its alignment, and how many columns it spans (`span` is a
/// multiple of `col_w` when a label spills into empty neighbours; FR-005).
fn cell_text(
    session: &Session,
    addr: CellAddr,
    value: &Value,
    col_w: u16,
    visible_index: u16,
    visible_cols: u16,
) -> (String, Alignment, u16) {
    let width = usize::from(col_w);
    match value {
        Value::Text(s) => {
            let align = match session.calc().content(addr) {
                Some(CellContent::Label { align, .. }) => *align,
                _ => Alignment::Left,
            };
            let mut span = col_w;
            if align == Alignment::Left {
                // Spill over following cells that are empty, until the first non-empty one.
                let mut next = visible_index + 1;
                while UnicodeWidthStr::width(s.as_str()) > usize::from(span)
                    && next < visible_cols
                    && addr.col() + u32::from(next - visible_index) <= MAX_COL
                {
                    let Some(neighbour) = CellAddr::new(addr.col() + u32::from(next - visible_index), addr.row()) else {
                        break;
                    };
                    if session.calc().value(neighbour) != Value::Empty {
                        break;
                    }
                    span = span.saturating_add(col_w);
                    next += 1;
                }
            }
            let shown: String = s.chars().take(usize::from(span)).collect();
            (shown, align, span)
        }
        other => (other.grid_text(width), Alignment::Right, col_w),
    }
}
