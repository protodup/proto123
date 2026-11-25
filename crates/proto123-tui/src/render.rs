//! Draws one frame of the 80×24 anatomy (docs/interaction-spec.md §1) and records every
//! drawn target in the `LayoutMap` in the same pass. All colors and metrics come from the
//! session's theme tokens.

use crate::layout::{ADDRESS_BOX_WIDTH, FOOTER_ROWS, HEADER_ROWS, meets_minimum};
use crate::layout_map::{HitTarget, LayoutMap};
use crate::style::{style, style_with_modifier};
use proto123_coord::{CellAddr, MAX_COL, MAX_ROW, col_letters};
use proto123_engine::menu::second_line_for_node;
use proto123_engine::{CATALOG, Mode, Prompt, PromptButton, Session, StringKey, text};
use proto123_model::{Alignment, CellContent};
use proto123_theme::GlyphSet;
use proto123_value::Value;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use unicode_width::UnicodeWidthStr;

/// Writes `s` at `(x, y)`, clipped to `area`.
fn put(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, st: ratatui::style::Style) {
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
    if let Some(edit) = session.edit() {
        return (edit.buffer.clone(), Some(edit.caret));
    }
    let sel = session.selection();
    if sel.start() != sel.end() {
        return (sel.to_string(), None);
    }
    (
        session
            .calc()
            .content(session.active())
            .map(|c| c.formula_bar_text())
            .unwrap_or_default(),
        None,
    )
}

/// Renders `session` into `area` and returns the targets it drew.
pub fn render(session: &Session, area: Rect, buf: &mut Buffer) -> LayoutMap {
    let theme = session.theme();
    let tokens = theme.tokens;
    let metrics = theme.metrics;
    let allow_bold = theme.allow_bold;
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
    let selection = session.selection();
    let has_multi_sel = selection.start() != selection.end();
    let vp = session.viewport();
    let row_w = metrics.row_header_width;
    let col_w = metrics.col_width;
    let two_line_context =
        matches!(session.mode(), Mode::Menu | Mode::Dialog) || session.root_hover().is_some();
    let extra_header: u16 = u16::from(two_line_context);
    let header_rows = HEADER_ROWS + extra_header;
    let grid_top = area.y + header_rows;
    let grid_rows = area.height.saturating_sub(header_rows + FOOTER_ROWS);
    let grid_cols = area.width.saturating_sub(row_w) / col_w.max(1);
    map.set_grid_rect(Rect::new(
        area.x + row_w,
        grid_top,
        grid_cols * col_w,
        grid_rows,
    ));
    let right_margin_x = area.x + row_w + grid_cols * col_w;
    if right_margin_x < area.right() {
        map.push(
            Rect::new(
                right_margin_x,
                grid_top,
                area.right() - right_margin_x,
                grid_rows,
            ),
            crate::layout_map::HitTarget::GridEdge(crate::layout_map::EdgeDir::Right),
        );
    }

    // Control line: address box, formula-bar text, mode badge.
    let control = Rect::new(area.x, area.y, area.width, 1);
    buf.set_style(control, style(tokens.control_line));
    let address = format!("[{active}]");
    put(
        buf,
        area,
        area.x,
        area.y,
        &format!("{address:<w$}", w = usize::from(ADDRESS_BOX_WIDTH)),
        style(tokens.header),
    );
    map.push(
        Rect::new(area.x, area.y, ADDRESS_BOX_WIDTH, 1),
        HitTarget::AddressBox,
    );

    let badge = text(session.mode().badge());
    let badge_w = u16::try_from(badge.len()).unwrap_or(0);
    let badge_x = area.right().saturating_sub(badge_w + 1);
    put(buf, area, badge_x, area.y, badge, style(tokens.badge));

    let (content, caret) = control_text(session);
    let content_x = area.x + ADDRESS_BOX_WIDTH + 1;
    let content_w = badge_x.saturating_sub(content_x + 1);
    let content_rect = Rect::new(content_x, area.y, content_w, 1);
    put(
        buf,
        area,
        content_x,
        area.y,
        &content,
        style(tokens.control_line),
    );
    map.push(content_rect, HitTarget::FormulaBar { char_index: 0 });
    if let Some(caret) = caret {
        let caret_x = content_x.saturating_add(u16::try_from(caret).unwrap_or(u16::MAX));
        if caret_x < badge_x
            && let Some(cell) = buf.cell_mut((caret_x, area.y))
        {
            cell.set_style(style_with_modifier(
                tokens.control_line,
                Modifier::REVERSED,
                allow_bold,
            ));
        }
    }

    // Row 1 (and Row 2 when MENU, DIALOG, or root hover is active).
    render_context_area(session, area, buf, &mut map);

    // Column headers (and corner glyph from theme.glyphs in border color).
    let header_y = area.y + 2 + extra_header;
    let corner = match theme.glyphs {
        GlyphSet::Box => "────┼",
        GlyphSet::Ascii => "----+",
    };
    put(buf, area, area.x, header_y, corner, style(tokens.border));
    for i in 0..grid_cols {
        let col = vp.left.saturating_add(u32::from(i));
        if col > MAX_COL {
            break;
        }
        let x = area.x + row_w + i * col_w;
        let letters = col_letters(col);
        let label = if col == active.col() {
            format!("[{letters}]")
        } else {
            letters
        };
        let st = if col == active.col() {
            style(tokens.active_cell)
        } else {
            style(tokens.header)
        };
        put(
            buf,
            area,
            x,
            header_y,
            &align_in(&label, usize::from(col_w), Alignment::Center),
            st,
        );
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
        let header = if is_active_row {
            format!("[{number}]")
        } else {
            format!(" {number} ")
        };
        let header_st = if is_active_row {
            style(tokens.active_cell)
        } else {
            style(tokens.header)
        };
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
            let is_selected = has_multi_sel && selection.contains(addr);
            let is_error = matches!(value, Value::Error(_));
            let cell_st = if is_active {
                style(tokens.active_cell)
            } else if is_selected {
                style(tokens.selection)
            } else if is_error {
                // Error tokens carry their own text, so the error token adds emphasis, not meaning (FR-035).
                style(tokens.error)
            } else {
                style(tokens.grid)
            };
            let (text_out, align, span) = cell_text(session, addr, &value, col_w, i, grid_cols);
            put(
                buf,
                area,
                x,
                y,
                &align_in(&text_out, usize::from(span), align),
                cell_st,
            );
            map.push(Rect::new(x, y, col_w, 1), HitTarget::GridCell(addr));
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
        let st = match status.kind {
            proto123_engine::StatusKind::Error => style(tokens.error),
            proto123_engine::StatusKind::Info => style(tokens.status),
        };
        put(buf, area, area.x, status_y, &status.message(), st);
    }
    if session.extend_mode() {
        let ext = text(StringKey::StatusExt);
        let ext_w = u16::try_from(ext.len()).unwrap_or(5);
        let ext_x = area.right().saturating_sub(ext_w + 1);
        put(buf, area, ext_x, status_y, ext, style(tokens.badge));
    }
    map
}

fn render_context_area(session: &Session, area: Rect, buf: &mut Buffer, map: &mut LayoutMap) {
    let tokens = session.theme().tokens;
    let allow_bold = session.theme().allow_bold;
    let y1 = area.y + 1;
    buf.set_style(Rect::new(area.x, y1, area.width, 1), style(tokens.menu));

    match session.mode() {
        Mode::Dialog => {
            let Some(prompt) = session.prompt() else {
                return;
            };
            let label = text(prompt.label_key());
            put(buf, area, area.x + 1, y1, label, style(tokens.menu));
            let label_w = u16::try_from(UnicodeWidthStr::width(label)).unwrap_or(0);
            let btn_y = area.y + 2;
            buf.set_style(Rect::new(area.x, btn_y, area.width, 1), style(tokens.menu));

            match prompt {
                Prompt::ConfirmNew { focus } => {
                    draw_button(
                        buf,
                        area,
                        map,
                        ButtonSpec {
                            x: area.x + 1,
                            y: btn_y,
                            label_key: StringKey::ButtonYes,
                            button: PromptButton::Yes,
                            focused: *focus == PromptButton::Yes,
                        },
                        tokens,
                    );
                    draw_button(
                        buf,
                        area,
                        map,
                        ButtonSpec {
                            x: area.x + 10,
                            y: btn_y,
                            label_key: StringKey::ButtonNo,
                            button: PromptButton::No,
                            focused: *focus == PromptButton::No,
                        },
                        tokens,
                    );
                }
                Prompt::RangeSource { .. } | Prompt::RangeDest { .. } => {
                    let field_x = area.x + 2 + label_w;
                    let field_w = area.right().saturating_sub(field_x + 1);
                    put(
                        buf,
                        area,
                        field_x,
                        y1,
                        prompt.text(),
                        style(tokens.control_line),
                    );
                    map.push(
                        Rect::new(field_x, y1, field_w.max(1), 1),
                        HitTarget::PromptField { char_index: 0 },
                    );
                    let caret_x =
                        field_x.saturating_add(u16::try_from(prompt.caret()).unwrap_or(u16::MAX));
                    if caret_x < area.right()
                        && let Some(cell) = buf.cell_mut((caret_x, y1))
                    {
                        cell.set_style(style_with_modifier(
                            tokens.control_line,
                            Modifier::REVERSED,
                            allow_bold,
                        ));
                    }
                    let ok_w = draw_button(
                        buf,
                        area,
                        map,
                        ButtonSpec {
                            x: area.x + 1,
                            y: btn_y,
                            label_key: StringKey::ButtonOk,
                            button: PromptButton::Ok,
                            focused: prompt.focus() == Some(PromptButton::Ok),
                        },
                        tokens,
                    );
                    draw_button(
                        buf,
                        area,
                        map,
                        ButtonSpec {
                            x: area.x + 1 + ok_w + 2,
                            y: btn_y,
                            label_key: StringKey::ButtonCancel,
                            button: PromptButton::Cancel,
                            focused: prompt.focus() == Some(PromptButton::Cancel),
                        },
                        tokens,
                    );
                }
            }
        }
        Mode::Menu => {
            if let Some(menu) = session.menu() {
                let level = menu.current_level(CATALOG);
                let depth = menu.path.len();
                let active_idx = menu.active_index();
                let mut x = area.x + 1;
                for (index, node) in level.iter().enumerate() {
                    let highlighted = index == active_idx;
                    let label = text(node.label);
                    let item = if highlighted {
                        format!(">{label}")
                    } else {
                        format!(" {label}")
                    };
                    let st = if highlighted {
                        style(tokens.menu_highlight)
                    } else {
                        style(tokens.menu)
                    };
                    let w = u16::try_from(UnicodeWidthStr::width(item.as_str())).unwrap_or(0);
                    put(buf, area, x, y1, &item, st);
                    map.push(
                        Rect::new(x, y1, w, 1),
                        HitTarget::MenuItem {
                            level: depth,
                            index,
                        },
                    );
                    x = x.saturating_add(w + 2);
                }
                let y2 = area.y + 2;
                buf.set_style(Rect::new(area.x, y2, area.width, 1), style(tokens.menu));
                let second = menu.second_line_text(CATALOG);
                put(buf, area, area.x + 1, y2, &second, style(tokens.menu));
            }
        }
        _ => {
            let hover = session.root_hover();
            let mut x = area.x + 1;
            for (index, node) in CATALOG.iter().enumerate() {
                let highlighted = hover == Some(index);
                let label = text(node.label);
                let item = if highlighted {
                    format!(">{label}")
                } else {
                    format!(" {label}")
                };
                let st = if highlighted {
                    style(tokens.menu_highlight)
                } else {
                    style(tokens.menu)
                };
                let w = u16::try_from(UnicodeWidthStr::width(item.as_str())).unwrap_or(0);
                put(buf, area, x, y1, &item, st);
                map.push(
                    Rect::new(x, y1, w, 1),
                    HitTarget::MenuItem { level: 0, index },
                );
                x = x.saturating_add(w + 2);
            }
            if let Some(idx) = hover
                && let Some(node) = CATALOG.get(idx)
            {
                let y2 = area.y + 2;
                buf.set_style(Rect::new(area.x, y2, area.width, 1), style(tokens.menu));
                put(
                    buf,
                    area,
                    area.x + 1,
                    y2,
                    &second_line_for_node(node),
                    style(tokens.menu),
                );
            }
        }
    }
}

struct ButtonSpec {
    x: u16,
    y: u16,
    label_key: StringKey,
    button: PromptButton,
    focused: bool,
}

fn draw_button(
    buf: &mut Buffer,
    area: Rect,
    map: &mut LayoutMap,
    spec: ButtonSpec,
    tokens: proto123_theme::ThemeTokens,
) -> u16 {
    let label = text(spec.label_key);
    let item = if spec.focused {
        format!(">[{label}]")
    } else {
        format!(" [{label}]")
    };
    let st = if spec.focused {
        style(tokens.menu_highlight)
    } else {
        style(tokens.menu)
    };
    let w = u16::try_from(UnicodeWidthStr::width(item.as_str())).unwrap_or(0);
    put(buf, area, spec.x, spec.y, &item, st);
    map.push(
        Rect::new(spec.x, spec.y, w, 1),
        HitTarget::PromptButton(spec.button),
    );
    w
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
                let mut next = visible_index + 1;
                while UnicodeWidthStr::width(s.as_str()) > usize::from(span)
                    && next < visible_cols
                    && addr.col() + u32::from(next - visible_index) <= MAX_COL
                {
                    let Some(neighbour) =
                        CellAddr::new(addr.col() + u32::from(next - visible_index), addr.row())
                    else {
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
