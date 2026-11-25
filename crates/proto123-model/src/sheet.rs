use crate::content::CellContent;
use proto123_coord::{CellAddr, CellRange};
use proto123_value::Value;
use std::collections::HashMap;

/// One populated grid cell: the user's content and its cached computed value.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub content: CellContent,
    /// Cached result. Formulas are filled in by the calc layer; constants are set on creation.
    pub value: Value,
}

impl Cell {
    pub fn new(content: CellContent) -> Self {
        let value = content.constant_value().unwrap_or(Value::Empty);
        Self { content, value }
    }
}

/// Sparse grid: an absent entry is `Empty`. Storing nothing removes the entry (FR-001).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sheet {
    cells: HashMap<CellAddr, Cell>,
}

impl Sheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cell(&self, addr: CellAddr) -> Option<&Cell> {
        self.cells.get(&addr)
    }

    pub fn value(&self, addr: CellAddr) -> Value {
        self.cells.get(&addr).map_or(Value::Empty, |c| c.value.clone())
    }

    /// Replaces the content of a cell. `None` erases it and removes the entry.
    pub fn set_content(&mut self, addr: CellAddr, content: Option<CellContent>) {
        match content {
            Some(content) => {
                self.cells.insert(addr, Cell::new(content));
            }
            None => {
                self.cells.remove(&addr);
            }
        }
    }

    /// Updates the cached value of a populated cell. Ignored for empty cells.
    pub fn set_value(&mut self, addr: CellAddr, value: Value) {
        if let Some(cell) = self.cells.get_mut(&addr) {
            cell.value = value;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn populated(&self) -> usize {
        self.cells.len()
    }

    /// Visits populated cells inside `range`, row-major. Walks the smaller of the range's
    /// address space and the populated set, so a huge range over a few cells is cheap.
    pub fn for_each_populated_in(&self, range: CellRange, mut f: impl FnMut(CellAddr, &Cell)) {
        if range.area() <= self.cells.len() as u64 {
            for addr in range.iter() {
                if let Some(cell) = self.cells.get(&addr) {
                    f(addr, cell);
                }
            }
        } else {
            let mut hits: Vec<(CellAddr, &Cell)> =
                self.cells.iter().filter(|(a, _)| range.contains(**a)).map(|(a, c)| (*a, c)).collect();
            hits.sort_unstable_by_key(|(a, _)| *a);
            for (addr, cell) in hits {
                f(addr, cell);
            }
        }
    }
}

/// A workbook holds exactly one sheet in this slice.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Workbook {
    pub sheet: Sheet,
}

impl Workbook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.sheet.is_empty()
    }
}
