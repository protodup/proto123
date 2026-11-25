//! Dependency graph: what each formula reads, and the reverse edges used to find dependents.

use proto123_coord::{CellAddr, CellRange};
use proto123_formula::Formula;
use std::collections::{HashMap, HashSet};

/// The cells and ranges one formula reads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Precedents {
    pub cells: Vec<CellAddr>,
    pub ranges: Vec<CellRange>,
}

#[derive(Clone, Debug, Default)]
pub struct DependencyGraph {
    precedents: HashMap<CellAddr, Precedents>,
    cell_dependents: HashMap<CellAddr, HashSet<CellAddr>>,
    range_dependents: Vec<(CellRange, CellAddr)>,
}

impl DependencyGraph {
    /// Replaces the precedents of `addr` (`None` removes the formula from the graph).
    pub fn set_formula(&mut self, addr: CellAddr, formula: Option<&Formula>) {
        self.remove(addr);
        let Some(formula) = formula else {
            return;
        };
        let (cells, ranges) = formula.references();
        let mut seen = HashSet::new();
        let cells: Vec<CellAddr> = cells.into_iter().filter(|c| seen.insert(*c)).collect();
        let mut seen_ranges = HashSet::new();
        let ranges: Vec<CellRange> = ranges.into_iter().filter(|r| seen_ranges.insert(*r)).collect();
        for cell in &cells {
            self.cell_dependents.entry(*cell).or_default().insert(addr);
        }
        for range in &ranges {
            self.range_dependents.push((*range, addr));
        }
        self.precedents.insert(addr, Precedents { cells, ranges });
    }

    fn remove(&mut self, addr: CellAddr) {
        let Some(old) = self.precedents.remove(&addr) else {
            return;
        };
        for cell in old.cells {
            if let Some(set) = self.cell_dependents.get_mut(&cell) {
                set.remove(&addr);
                if set.is_empty() {
                    self.cell_dependents.remove(&cell);
                }
            }
        }
        self.range_dependents.retain(|(_, f)| *f != addr);
    }

    /// Distinct formula cells that read `addr` directly or through a range.
    pub fn dependents_of(&self, addr: CellAddr) -> Vec<CellAddr> {
        let mut out: HashSet<CellAddr> = self.cell_dependents.get(&addr).cloned().unwrap_or_default();
        for (range, formula) in &self.range_dependents {
            if range.contains(addr) {
                out.insert(*formula);
            }
        }
        let mut list: Vec<CellAddr> = out.into_iter().collect();
        list.sort_unstable();
        list
    }

    pub fn precedents(&self, addr: CellAddr) -> Option<&Precedents> {
        self.precedents.get(&addr)
    }
}
