//! Entry classification (contracts/formula-grammar.md §1): turns the committed buffer
//! into the cell content it means, or a syntax error with its character position.

use proto123_formula::Formula;
use proto123_model::{Alignment, CellContent};

/// A rejected entry: the character index in the buffer and whether the quote hint applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryError {
    pub pos: usize,
    /// True when the entry did not start with `=`, `+`, or `@`, so an apostrophe would fix it.
    pub quote_hint: bool,
}

/// `^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$`, checked byte by byte.
fn is_plain_number(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    if matches!(b.first(), Some(b'+' | b'-')) {
        i += 1;
    }
    let int_start = i;
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let int_digits = i - int_start;
    let mut frac_digits = 0;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let frac_start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        frac_digits = i - frac_start;
    }
    if int_digits == 0 && frac_digits == 0 {
        return false;
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let exp_start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == exp_start {
            return false;
        }
    }
    i == b.len()
}

/// Classifies a committed buffer. `Ok(None)` means erase the cell.
pub fn classify_entry(buffer: &str) -> Result<Option<CellContent>, EntryError> {
    if buffer.is_empty() {
        return Ok(None);
    }
    let trimmed = buffer.trim();
    if is_plain_number(trimmed) {
        return match trimmed.parse::<f64>() {
            Ok(n) if n.is_finite() => Ok(Some(CellContent::Number(n))),
            _ => Err(EntryError { pos: 0, quote_hint: false }),
        };
    }

    let lead = buffer.trim_start();
    let lead_offset = buffer.chars().count() - lead.chars().count();
    let first = lead.chars().next();

    if let Some(prefix @ ('\'' | '"' | '^')) = first {
        let align = match prefix {
            '\'' => Alignment::Left,
            '"' => Alignment::Right,
            _ => Alignment::Center,
        };
        let text: String = lead.chars().skip(1).collect();
        return Ok(Some(CellContent::Label { text, align, prefix: Some(prefix) }));
    }

    let is_formula_start = matches!(first, Some('=' | '+' | '-' | '.' | '(' | '@')) || first.is_some_and(|c| c.is_ascii_digit());
    if is_formula_start {
        let quote_hint = !matches!(first, Some('=' | '+' | '@'));
        let stripped = usize::from(matches!(first, Some('=' | '+')));
        let body: String = lead.chars().skip(stripped).collect();
        return match Formula::parse(&body) {
            Ok(formula) => Ok(Some(CellContent::Formula(formula))),
            Err(e) => Err(EntryError { pos: lead_offset + stripped + e.pos, quote_hint }),
        };
    }

    Ok(Some(CellContent::Label { text: buffer.to_owned(), align: Alignment::Left, prefix: None }))
}
