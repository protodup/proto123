//! L1 formula language: lexer, Pratt parser, AST, canonical OpenFormula-style printer,
//! reference extraction, and the evaluator over an abstract [`CellSource`].

mod ast;
mod eval;
mod lexer;
mod parser;
mod print;
mod refs;
mod rewrite;

pub use ast::{BinOp, Expr, UnOp};
pub use eval::{CellSource, evaluate};
pub use parser::{MAX_DEPTH, MAX_LEN, ParseError, ParseErrorKind};
pub use rewrite::RefRemap;

use proto123_coord::{CellRange, RefAddr};

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

    /// Returns a copy with relative references shifted by `(dc, dr)` and canonical text updated.
    pub fn shifted(&self, dc: i64, dr: i64) -> Self {
        Self::from_expr(self.expr.shifted(dc, dr))
    }

    /// Returns a copy with references rewritten by `f` and canonical text updated.
    pub fn remap(&self, f: &dyn Fn(RefAddr) -> RefRemap) -> Self {
        Self::from_expr(self.expr.remap(f))
    }

    /// Returns a copy with references rewritten for a Range Move (FR-020).
    pub fn remap_move(&self, src: CellRange, dc: i64, dr: i64, dest: CellRange) -> Self {
        Self::from_expr(self.expr.remap_move(src, dc, dr, dest))
    }
}
