use std::fmt;

/// Last 0-based column index (`XFD`).
pub const MAX_COL: u32 = 16_383;
/// Last 0-based row index (row `1048576`).
pub const MAX_ROW: u32 = 1_048_575;

/// A cell position on the grid. Ordering is row-major (row first, then column).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellAddr {
    row: u32,
    col: u32,
}

impl CellAddr {
    /// `A1`.
    pub const ORIGIN: Self = Self { row: 0, col: 0 };

    /// Returns `None` when the position lies outside the grid.
    pub fn new(col: u32, row: u32) -> Option<Self> {
        (col <= MAX_COL && row <= MAX_ROW).then_some(Self { row, col })
    }

    pub fn col(self) -> u32 {
        self.col
    }

    pub fn row(self) -> u32 {
        self.row
    }

    /// Moves by a signed offset; `None` if the result leaves the grid.
    pub fn offset(self, dc: i64, dr: i64) -> Option<Self> {
        let col = i64::from(self.col).checked_add(dc)?;
        let row = i64::from(self.row).checked_add(dr)?;
        Self::new(u32::try_from(col).ok()?, u32::try_from(row).ok()?)
    }
}

impl fmt::Display for CellAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", col_letters(self.col), self.row + 1)
    }
}

/// Spreadsheet column letters for a 0-based column index (`0` → `A`, `26` → `AA`).
pub fn col_letters(col: u32) -> String {
    let mut n = col + 1;
    let mut out = Vec::new();
    while n > 0 {
        let rem = (n - 1) % 26;
        out.push(char::from(b'A' + rem as u8));
        n = (n - 1) / 26;
    }
    out.iter().rev().collect()
}

/// Parses column letters (case-insensitive, 1–3 letters) into a 0-based column index.
/// Returns `None` for empty input, non-letters, or a column beyond `XFD`.
pub fn letters_to_col(text: &str) -> Option<u32> {
    if text.is_empty() || text.len() > 3 {
        return None;
    }
    let mut n: u32 = 0;
    for ch in text.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        n = n * 26 + (ch.to_ascii_uppercase() as u32 - 'A' as u32 + 1);
    }
    let col = n.checked_sub(1)?;
    (col <= MAX_COL).then_some(col)
}
