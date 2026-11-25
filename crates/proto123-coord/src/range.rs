use crate::addr::CellAddr;
use crate::refaddr::RefAddr;
use std::fmt;

/// A rectangular block of cells, always normalized so `start <= end` on both axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellRange {
    start: CellAddr,
    end: CellAddr,
}

impl CellRange {
    /// Builds a range from any two corners.
    pub fn new(a: CellAddr, b: CellAddr) -> Self {
        Self {
            start: CellAddr::new(a.col().min(b.col()), a.row().min(b.row())).unwrap_or(a),
            end: CellAddr::new(a.col().max(b.col()), a.row().max(b.row())).unwrap_or(b),
        }
    }

    /// Parses `A1:B3` in any corner order. Absolute markers are ignored.
    pub fn parse(text: &str) -> Option<Self> {
        let (left, right) = text.split_once(':')?;
        let a = RefAddr::parse(left.trim())?;
        let b = RefAddr::parse(right.trim())?;
        Some(Self::new(a.addr, b.addr))
    }

    pub fn start(&self) -> CellAddr {
        self.start
    }

    pub fn end(&self) -> CellAddr {
        self.end
    }

    pub fn width(&self) -> u32 {
        self.end.col() - self.start.col() + 1
    }

    pub fn height(&self) -> u32 {
        self.end.row() - self.start.row() + 1
    }

    pub fn area(&self) -> u64 {
        u64::from(self.width()) * u64::from(self.height())
    }

    pub fn contains(&self, addr: CellAddr) -> bool {
        (self.start.col()..=self.end.col()).contains(&addr.col())
            && (self.start.row()..=self.end.row()).contains(&addr.row())
    }

    /// Every address in the range, row-major.
    pub fn iter(&self) -> impl Iterator<Item = CellAddr> {
        let (c0, c1) = (self.start.col(), self.end.col());
        let (r0, r1) = (self.start.row(), self.end.row());
        (r0..=r1).flat_map(move |r| (c0..=c1).filter_map(move |c| CellAddr::new(c, r)))
    }
}

impl fmt::Display for CellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}
