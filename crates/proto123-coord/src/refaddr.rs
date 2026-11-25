use crate::addr::{CellAddr, MAX_ROW, col_letters, letters_to_col};
use std::fmt;

/// A reference as written in a formula: `A1`, `$A1`, `A$1`, or `$A$1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RefAddr {
    pub addr: CellAddr,
    pub col_abs: bool,
    pub row_abs: bool,
}

/// Outcome of reading a token as a reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefSyntax {
    /// A reference inside the grid.
    Valid(RefAddr),
    /// Reference-shaped (letters then digits) but outside the grid, e.g. `XFE1` or `A0`.
    OutOfGrid,
    /// Not reference-shaped at all.
    NotARef,
}

impl RefAddr {
    /// Classifies a token. Out-of-grid references are distinguished so the
    /// formula parser can produce `#REF!` instead of a syntax error.
    pub fn classify(text: &str) -> RefSyntax {
        let bytes = text.as_bytes();
        let mut i = 0;
        let col_abs = bytes.first() == Some(&b'$');
        if col_abs {
            i += 1;
        }
        let letters_start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_alphabetic) {
            i += 1;
        }
        let letters = text.get(letters_start..i).unwrap_or("");
        if letters.is_empty() || letters.len() > 3 {
            return RefSyntax::NotARef;
        }
        let row_abs = bytes.get(i) == Some(&b'$');
        if row_abs {
            i += 1;
        }
        let digits = text.get(i..).unwrap_or("");
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return RefSyntax::NotARef;
        }
        let row = match digits.parse::<u64>() {
            Ok(r) if (1..=u64::from(MAX_ROW) + 1).contains(&r) => (r - 1) as u32,
            _ => return RefSyntax::OutOfGrid,
        };
        let Some(col) = letters_to_col(letters) else {
            return RefSyntax::OutOfGrid;
        };
        match CellAddr::new(col, row) {
            Some(addr) => RefSyntax::Valid(RefAddr { addr, col_abs, row_abs }),
            None => RefSyntax::OutOfGrid,
        }
    }

    /// Parses a reference, returning `None` when the text is not a valid in-grid reference.
    pub fn parse(text: &str) -> Option<Self> {
        match Self::classify(text) {
            RefSyntax::Valid(r) => Some(r),
            _ => None,
        }
    }

    /// Shifts a relative reference by `(dc, dr)`. Absolute parts are unchanged.
    /// `None` when the result leaves the grid.
    pub fn shifted(self, dc: i64, dr: i64) -> Option<Self> {
        let dc = if self.col_abs { 0 } else { dc };
        let dr = if self.row_abs { 0 } else { dr };
        Some(Self { addr: self.addr.offset(dc, dr)?, ..self })
    }
}

impl fmt::Display for RefAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dollar_c = if self.col_abs { "$" } else { "" };
        let dollar_r = if self.row_abs { "$" } else { "" };
        write!(
            f,
            "{dollar_c}{}{dollar_r}{}",
            col_letters(self.addr.col()),
            self.addr.row() + 1
        )
    }
}
