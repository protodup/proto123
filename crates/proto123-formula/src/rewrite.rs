use crate::ast::Expr;
use proto123_coord::{CellRange, RefAddr};
use proto123_value::ErrorKind;

/// How a single cell reference is rewritten during a range move or custom remap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefRemap {
    Keep,
    To(RefAddr),
    Invalid,
}

impl Expr {
    /// Shifts every relative reference by `(dc, dr)` (used by Range Copy).
    /// Absolute `$` parts stay fixed; any reference shifted off the grid becomes `#REF!`.
    pub fn shifted(&self, dc: i64, dr: i64) -> Self {
        match self {
            Self::Number(_) | Self::Bool(_) | Self::ErrorLit(_) => self.clone(),
            Self::Ref(r) => r
                .shifted(dc, dr)
                .map_or(Self::ErrorLit(ErrorKind::Ref), Self::Ref),
            Self::Range(a, b) => match (a.shifted(dc, dr), b.shifted(dc, dr)) {
                (Some(a2), Some(b2)) => Self::Range(a2, b2),
                _ => Self::ErrorLit(ErrorKind::Ref),
            },
            Self::Unary(op, inner) => Self::Unary(*op, Box::new(inner.shifted(dc, dr))),
            Self::Binary(op, lhs, rhs) => Self::Binary(
                *op,
                Box::new(lhs.shifted(dc, dr)),
                Box::new(rhs.shifted(dc, dr)),
            ),
            Self::Call { name, args } => Self::Call {
                name: name.clone(),
                args: args.iter().map(|a| a.shifted(dc, dr)).collect(),
            },
        }
    }

    /// Rewrites every reference using `f`. `Invalid` turns the reference or range into `#REF!`.
    pub fn remap(&self, f: &dyn Fn(RefAddr) -> RefRemap) -> Self {
        match self {
            Self::Number(_) | Self::Bool(_) | Self::ErrorLit(_) => self.clone(),
            Self::Ref(r) => match f(*r) {
                RefRemap::Keep => Self::Ref(*r),
                RefRemap::To(n) => Self::Ref(n),
                RefRemap::Invalid => Self::ErrorLit(ErrorKind::Ref),
            },
            Self::Range(a, b) => {
                let ra = match f(*a) {
                    RefRemap::Keep => Some(*a),
                    RefRemap::To(n) => Some(n),
                    RefRemap::Invalid => None,
                };
                let rb = match f(*b) {
                    RefRemap::Keep => Some(*b),
                    RefRemap::To(n) => Some(n),
                    RefRemap::Invalid => None,
                };
                match (ra, rb) {
                    (Some(a2), Some(b2)) => Self::Range(a2, b2),
                    _ => Self::ErrorLit(ErrorKind::Ref),
                }
            }
            Self::Unary(op, inner) => Self::Unary(*op, Box::new(inner.remap(f))),
            Self::Binary(op, lhs, rhs) => {
                Self::Binary(*op, Box::new(lhs.remap(f)), Box::new(rhs.remap(f)))
            }
            Self::Call { name, args } => Self::Call {
                name: name.clone(),
                args: args.iter().map(|a| a.remap(f)).collect(),
            },
        }
    }

    /// Rewrites references for a Range Move from `src` by `(dc, dr)` overwriting `dest` (FR-020).
    /// - Single refs inside `src` follow by `(dc, dr)` regardless of `$` flags.
    /// - Single refs to `dest` outside `src` (overwritten cells) become `#REF!`.
    /// - Range refs follow only when the entire range lies inside `src`; otherwise unchanged.
    pub fn remap_move(&self, src: CellRange, dc: i64, dr: i64, dest: CellRange) -> Self {
        match self {
            Self::Number(_) | Self::Bool(_) | Self::ErrorLit(_) => self.clone(),
            Self::Ref(r) => {
                if src.contains(r.addr) {
                    r.addr
                        .offset(dc, dr)
                        .map_or(Self::ErrorLit(ErrorKind::Ref), |addr| {
                            Self::Ref(RefAddr { addr, ..*r })
                        })
                } else if dest.contains(r.addr) {
                    Self::ErrorLit(ErrorKind::Ref)
                } else {
                    Self::Ref(*r)
                }
            }
            Self::Range(a, b) => {
                let rng = CellRange::new(a.addr, b.addr);
                if src.contains(rng.start()) && src.contains(rng.end()) {
                    match (a.addr.offset(dc, dr), b.addr.offset(dc, dr)) {
                        (Some(na), Some(nb)) => {
                            Self::Range(RefAddr { addr: na, ..*a }, RefAddr { addr: nb, ..*b })
                        }
                        _ => Self::ErrorLit(ErrorKind::Ref),
                    }
                } else {
                    Self::Range(*a, *b)
                }
            }
            Self::Unary(op, inner) => {
                Self::Unary(*op, Box::new(inner.remap_move(src, dc, dr, dest)))
            }
            Self::Binary(op, lhs, rhs) => Self::Binary(
                *op,
                Box::new(lhs.remap_move(src, dc, dr, dest)),
                Box::new(rhs.remap_move(src, dc, dr, dest)),
            ),
            Self::Call { name, args } => Self::Call {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| a.remap_move(src, dc, dr, dest))
                    .collect(),
            },
        }
    }
}
