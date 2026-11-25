//! Evaluator (data-model.md "Evaluation rules"). Iterative over the cell source; recursion is
//! bounded by the parser's nesting cap, never by sheet size.

use crate::ast::{BinOp, Expr, UnOp};
use proto123_coord::{CellAddr, CellRange};
use proto123_value::{ErrorKind, Value};

/// Read access to the sheet. `for_each_in_range` must visit only populated cells,
/// in row-major order, so `=SUM(A1:XFD1048576)` costs the number of populated cells.
pub trait CellSource {
    fn value(&self, addr: CellAddr) -> Value;
    fn for_each_in_range(&self, range: CellRange, f: &mut dyn FnMut(CellAddr, &Value));
}

/// Evaluates a formula. A formula that yields an empty reference reads as `0`.
pub fn evaluate(expr: &Expr, src: &dyn CellSource) -> Value {
    match eval(expr, src) {
        Value::Empty => Value::Number(0.0),
        other => other,
    }
}

fn eval(expr: &Expr, src: &dyn CellSource) -> Value {
    match expr {
        Expr::Number(n) => Value::number(*n),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::ErrorLit(kind) => Value::Error(*kind),
        Expr::Ref(r) => src.value(r.addr),
        // A range is only meaningful as an aggregate argument.
        Expr::Range(_, _) => Value::Error(ErrorKind::Value),
        Expr::Unary(op, operand) => {
            let x = match scalar(eval(operand, src)) {
                Ok(x) => x,
                Err(e) => return Value::Error(e),
            };
            match op {
                UnOp::Neg => Value::number(-x),
                UnOp::Plus => Value::number(x),
            }
        }
        Expr::Binary(op, lhs, rhs) => {
            let a = match scalar(eval(lhs, src)) {
                Ok(x) => x,
                Err(e) => return Value::Error(e),
            };
            let b = match scalar(eval(rhs, src)) {
                Ok(x) => x,
                Err(e) => return Value::Error(e),
            };
            binary(*op, a, b)
        }
        Expr::Call { name, args } => call(name, args, src),
    }
}

/// Coerces a value to a number for arithmetic: Empty → 0, Bool → 1/0, Text → `#VALUE!`.
fn scalar(v: Value) -> Result<f64, ErrorKind> {
    match v {
        Value::Empty => Ok(0.0),
        Value::Number(n) => Ok(n),
        Value::Bool(b) => Ok(if b { 1.0 } else { 0.0 }),
        Value::Text(_) => Err(ErrorKind::Value),
        Value::Error(e) => Err(e),
    }
}

fn binary(op: BinOp, a: f64, b: f64) -> Value {
    match op {
        BinOp::Add => Value::number(a + b),
        BinOp::Sub => Value::number(a - b),
        BinOp::Mul => Value::number(a * b),
        BinOp::Div => {
            if b == 0.0 {
                Value::Error(ErrorKind::Div0)
            } else {
                Value::number(a / b)
            }
        }
        BinOp::Pow => Value::number(a.powf(b)),
    }
}

#[derive(Clone, Copy)]
enum Agg {
    Sum,
    Average,
    Min,
    Max,
    Count,
}

#[derive(Default)]
struct Acc {
    sum: f64,
    count: u64,
    min: Option<f64>,
    max: Option<f64>,
    /// The first error seen, left to right (range cells in row-major order).
    err: Option<ErrorKind>,
}

impl Acc {
    fn push(&mut self, n: f64) {
        self.sum += n;
        self.count += 1;
        self.min = Some(self.min.map_or(n, |m| m.min(n)));
        self.max = Some(self.max.map_or(n, |m| m.max(n)));
    }

    fn note_error(&mut self, e: ErrorKind) {
        if self.err.is_none() {
            self.err = Some(e);
        }
    }

    fn finish(self, agg: Agg) -> Value {
        if let Some(e) = self.err {
            return Value::Error(e);
        }
        match agg {
            Agg::Sum => Value::number(self.sum),
            Agg::Average => {
                if self.count == 0 {
                    Value::Error(ErrorKind::Div0)
                } else {
                    Value::number(self.sum / self.count as f64)
                }
            }
            Agg::Min => Value::number(self.min.unwrap_or(0.0)),
            Agg::Max => Value::number(self.max.unwrap_or(0.0)),
            Agg::Count => Value::number(self.count as f64),
        }
    }
}

fn call(name: &str, args: &[Expr], src: &dyn CellSource) -> Value {
    let agg = match name {
        "SUM" => Agg::Sum,
        "AVERAGE" => Agg::Average,
        "MIN" => Agg::Min,
        "MAX" => Agg::Max,
        "COUNT" => Agg::Count,
        _ => return Value::Error(ErrorKind::Name),
    };
    if args.is_empty() {
        return Value::Error(ErrorKind::Value);
    }
    let mut acc = Acc::default();
    for arg in args {
        match arg {
            Expr::Range(a, b) => {
                let range = CellRange::new(a.addr, b.addr);
                src.for_each_in_range(range, &mut |_, v| match v {
                    Value::Number(n) => acc.push(*n),
                    Value::Error(e) => acc.note_error(*e),
                    // Text, Bool, and Empty cells inside a range are skipped.
                    _ => {}
                });
            }
            other => match eval(other, src) {
                Value::Number(n) => acc.push(n),
                Value::Bool(b) => acc.push(if b { 1.0 } else { 0.0 }),
                Value::Empty => {}
                Value::Error(e) => acc.note_error(e),
                // A direct text argument is a type error, not a skipped value.
                Value::Text(_) => acc.note_error(ErrorKind::Value),
            },
        }
    }
    acc.finish(agg)
}
