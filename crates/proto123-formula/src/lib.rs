//! L1 formula language: lexer, Pratt parser, AST, canonical OpenFormula-style printer,
//! reference extraction, and the evaluator over an abstract [`CellSource`].

mod ast;
mod eval;
mod lexer;
mod parser;
mod print;
mod refs;

pub use ast::{BinOp, Expr, UnOp};
pub use eval::{CellSource, evaluate};
pub use parser::{MAX_DEPTH, MAX_LEN, ParseError, ParseErrorKind};

/// A parsed formula. `canonical` always starts with `=` and re-parses to an equal `expr`.
#[derive(Clone, Debug, PartialEq)]
pub struct Formula {
    pub expr: Expr,
    pub canonical: String,
}

impl Formula {
    /// Parses a formula body (the text after the leading `=`, `+`, or `@`).
    pub fn parse(body: &str) -> Result<Self, ParseError> {
        let expr = parser::parse_body(body)?;
        let canonical = format!("={}", print::print(&expr));
        Ok(Self { expr, canonical })
    }

    /// Builds a formula from an expression, computing its canonical text.
    pub fn from_expr(expr: Expr) -> Self {
        let canonical = format!("={}", print::print(&expr));
        Self { expr, canonical }
    }
}
