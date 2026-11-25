//! Canonical printer (contracts/formula-grammar.md §3): minimal parentheses from the same
//! binding powers the parser uses, so `parse(print(e)) == e`.

use crate::ast::{Expr, PREFIX_BP, UnOp};
use proto123_value::format_full;

/// Binding power of an atom: never needs parentheses.
const ATOM_BP: u8 = 100;

pub(crate) fn print(expr: &Expr) -> String {
    print_expr(expr)
}

fn bp_of(expr: &Expr) -> u8 {
    match expr {
        Expr::Binary(op, _, _) => op.binding_power().0,
        Expr::Unary(_, _) => PREFIX_BP,
        _ => ATOM_BP,
    }
}

/// Prints `child`, wrapping it in parentheses when its binding power is below `required`.
fn print_child(child: &Expr, required: u8) -> String {
    let text = print_expr(child);
    if bp_of(child) < required {
        format!("({text})")
    } else {
        text
    }
}

fn print_expr(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => format_full(*n),
        Expr::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_owned(),
        Expr::ErrorLit(kind) => kind.token().to_owned(),
        Expr::Ref(r) => r.to_string(),
        Expr::Range(a, b) => format!("{a}:{b}"),
        Expr::Unary(op, operand) => {
            let sign = match op {
                UnOp::Neg => "-",
                UnOp::Plus => "+",
            };
            format!("{sign}{}", print_child(operand, PREFIX_BP))
        }
        Expr::Binary(op, lhs, rhs) => {
            let (left_bp, right_bp) = op.binding_power();
            // Left-assoc: the left operand may share the operator's power; the right may not.
            format!(
                "{}{}{}",
                print_child(lhs, left_bp),
                op.symbol(),
                print_child(rhs, right_bp)
            )
        }
        Expr::Call { name, args } => {
            let parts: Vec<String> = args.iter().map(print_expr).collect();
            format!("{name}({})", parts.join(","))
        }
    }
}
