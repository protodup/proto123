//! L0 value types: numbers, text, booleans, error tokens, and FR-006 display formatting.

mod display;

pub use display::{format_full, format_general};

use std::fmt;

/// Spreadsheet error kinds with their display tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Div0,
    Ref,
    Name,
    Value,
    Circ,
}

impl ErrorKind {
    pub fn token(self) -> &'static str {
        match self {
            Self::Div0 => "#DIV/0!",
            Self::Ref => "#REF!",
            Self::Name => "#NAME?",
            Self::Value => "#VALUE!",
            Self::Circ => "#CIRC!",
        }
    }
}

/// A computed or entered cell value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Empty,
    /// Always finite (see [`Value::number`]).
    Number(f64),
    Text(String),
    Bool(bool),
    Error(ErrorKind),
}

impl Value {
    /// Wraps a float, mapping non-finite results to `#VALUE!` so the invariant holds.
    pub fn number(n: f64) -> Self {
        if n.is_finite() {
            Self::Number(n)
        } else {
            Self::Error(ErrorKind::Value)
        }
    }

    /// Text shown in the grid for this value at the given cell width (FR-005, FR-006).
    pub fn grid_text(&self, width: usize) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Number(n) => format_general(*n, width),
            Self::Text(s) => s.clone(),
            Self::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_owned(),
            Self::Error(e) => e.token().to_owned(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => f.write_str(&format_full(*n)),
            other => f.write_str(&other.grid_text(usize::MAX)),
        }
    }
}
