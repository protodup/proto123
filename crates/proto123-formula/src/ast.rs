use proto123_coord::RefAddr;
use proto123_value::ErrorKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Plus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl BinOp {
    /// Binding power of the operator: `(left, right)`. Every operator is left-associative.
    pub(crate) fn binding_power(self) -> (u8, u8) {
        match self {
            Self::Add | Self::Sub => (10, 11),
            Self::Mul | Self::Div => (20, 21),
            Self::Pow => (30, 31),
        }
    }

    pub(crate) fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
            Self::Div => '/',
            Self::Pow => '^',
        }
    }
}

/// Binding power of prefix `+`/`-`: tighter than `^`, so `-2^2` is `(-2)^2`... see R8.
pub(crate) const PREFIX_BP: u8 = 40;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    Bool(bool),
    ErrorLit(ErrorKind),
    Ref(RefAddr),
    Range(RefAddr, RefAddr),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `name` is stored upper-case.
    Call {
        name: String,
        args: Vec<Expr>,
    },
}
