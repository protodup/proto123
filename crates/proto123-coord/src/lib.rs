//! L0 primitives: cell addresses, references as written in formulas, and ranges.

mod addr;
mod range;
mod refaddr;

pub use addr::{CellAddr, MAX_COL, MAX_ROW, col_letters, letters_to_col};
pub use range::CellRange;
pub use refaddr::{RefAddr, RefSyntax};
