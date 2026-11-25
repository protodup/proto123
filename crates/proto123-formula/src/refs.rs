//! Reference extraction for the dependency graph (`Formula::references`).

use crate::Formula;
use crate::ast::Expr;
use proto123_coord::{CellAddr, CellRange};

impl Formula {
    /// Every single-cell and range reference the formula reads, in source order.
    pub fn references(&self) -> (Vec<CellAddr>, Vec<CellRange>) {
        let mut cells = Vec::new();
        let mut ranges = Vec::new();
        collect(&self.expr, &mut cells, &mut ranges);
        (cells, ranges)
    }
}

fn collect(expr: &Expr, cells: &mut Vec<CellAddr>, ranges: &mut Vec<CellRange>) {
    match expr {
        Expr::Ref(r) => cells.push(r.addr),
        Expr::Range(a, b) => ranges.push(CellRange::new(a.addr, b.addr)),
        Expr::Unary(_, operand) => collect(operand, cells, ranges),
        Expr::Binary(_, lhs, rhs) => {
            collect(lhs, cells, ranges);
            collect(rhs, cells, ranges);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect(arg, cells, ranges);
            }
        }
        Expr::Number(_) | Expr::Bool(_) | Expr::ErrorLit(_) => {}
    }
}
