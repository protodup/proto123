use proto123_formula::Formula;
use proto123_value::{Value, format_full};

/// Horizontal alignment of a label. Numbers are always right-aligned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// What the user entered into a cell (the raw input, not the computed value).
#[derive(Clone, Debug, PartialEq)]
pub enum CellContent {
    /// Always finite; the engine rejects non-finite entries before they reach here.
    Number(f64),
    /// `prefix` is the alignment prefix the user typed (`'`, `"`, `^`), kept for the formula bar.
    Label {
        text: String,
        align: Alignment,
        prefix: Option<char>,
    },
    Formula(Formula),
}

impl CellContent {
    /// The value of a constant. `None` for formulas, which are computed by the calc layer.
    pub fn constant_value(&self) -> Option<Value> {
        match self {
            Self::Number(n) => Some(Value::Number(*n)),
            Self::Label { text, .. } => Some(Value::Text(text.clone())),
            Self::Formula(_) => None,
        }
    }

    /// Text the formula bar shows for this content (contracts/formula-grammar.md).
    pub fn formula_bar_text(&self) -> String {
        match self {
            Self::Number(n) => format_full(*n),
            Self::Label { text, prefix, .. } => match prefix {
                Some(p) => format!("{p}{text}"),
                None => text.clone(),
            },
            Self::Formula(f) => f.canonical.clone(),
        }
    }
}
