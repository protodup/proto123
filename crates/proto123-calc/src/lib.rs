//! L2 calculation: the workbook plus its dependency graph. `CalcSheet::apply` is the only
//! writer of cell contents; it recalculates the dirty subgraph once, in dependency order.

mod graph;

pub use graph::{DependencyGraph, Precedents};

use proto123_coord::{CellAddr, CellRange};
use proto123_formula::{CellSource, evaluate};
use proto123_model::{CellContent, Sheet, Workbook};
use proto123_value::{ErrorKind, Value};
use std::collections::{HashMap, HashSet, VecDeque};

/// What one `apply` evaluated and what changed on screen.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecalcReport {
    /// Formula cells recalculated (changed formulas and their transitive dependents),
    /// each once, in dependency order; cycle members are listed last. Its length is the
    /// SC-003 count.
    pub evaluated: Vec<CellAddr>,
    /// Cells, including changed constants, whose computed value differs from before.
    pub changed_display: Vec<CellAddr>,
}

#[derive(Clone, Debug, Default)]
pub struct CalcSheet {
    workbook: Workbook,
    graph: DependencyGraph,
}

/// Reads the sheet for the evaluator without copying it.
struct SheetSource<'a>(&'a Sheet);

impl CellSource for SheetSource<'_> {
    fn value(&self, addr: CellAddr) -> Value {
        self.0.value(addr)
    }

    fn for_each_in_range(&self, range: CellRange, f: &mut dyn FnMut(CellAddr, &Value)) {
        self.0
            .for_each_populated_in(range, |addr, cell| f(addr, &cell.value));
    }
}

impl CalcSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workbook(&self) -> &Workbook {
        &self.workbook
    }

    pub fn sheet(&self) -> &Sheet {
        &self.workbook.sheet
    }

    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    pub fn value(&self, addr: CellAddr) -> Value {
        self.workbook.sheet.value(addr)
    }

    pub fn content(&self, addr: CellAddr) -> Option<&CellContent> {
        self.workbook.sheet.cell(addr).map(|c| &c.content)
    }

    /// Applies content changes (`None` erases), updates the graph, and recalculates every
    /// formula downstream of a change exactly once.
    pub fn apply(&mut self, changes: &[(CellAddr, Option<CellContent>)]) -> RecalcReport {
        // Pre-change values, so the report compares against what was on screen.
        let mut old_values: HashMap<CellAddr, Value> = HashMap::new();
        let mut changed: Vec<CellAddr> = Vec::new();
        for (addr, content) in changes {
            if !old_values.contains_key(addr) {
                old_values.insert(*addr, self.value(*addr));
                changed.push(*addr);
            }
            let formula = match content {
                Some(CellContent::Formula(f)) => Some(f),
                _ => None,
            };
            self.graph.set_formula(*addr, formula);
            self.workbook.sheet.set_content(*addr, content.clone());
        }

        let (dirty, successors) = self.collect_dirty(&changed);
        let order = topo_order(&dirty, &successors);
        let processed: HashSet<CellAddr> = order.iter().copied().collect();

        let mut report = RecalcReport::default();
        for addr in &order {
            let before = old_values
                .get(addr)
                .cloned()
                .unwrap_or_else(|| self.value(*addr));
            let after = match self.content(*addr) {
                Some(CellContent::Formula(f)) => {
                    let v = evaluate(&f.expr, &SheetSource(&self.workbook.sheet));
                    self.workbook.sheet.set_value(*addr, v.clone());
                    report.evaluated.push(*addr);
                    v
                }
                Some(_) => self.value(*addr),
                None => Value::Empty,
            };
            if before != after {
                report.changed_display.push(*addr);
            }
        }

        // Formulas never reached by Kahn sit on a cycle or downstream of one: `#CIRC!` (FR-014).
        let mut unreached: Vec<CellAddr> = dirty
            .iter()
            .copied()
            .filter(|a| {
                !processed.contains(a) && matches!(self.content(*a), Some(CellContent::Formula(_)))
            })
            .collect();
        unreached.sort_unstable();
        for addr in unreached {
            let before = old_values
                .get(&addr)
                .cloned()
                .unwrap_or_else(|| self.value(addr));
            let circ = Value::Error(ErrorKind::Circ);
            self.workbook.sheet.set_value(addr, circ.clone());
            report.evaluated.push(addr);
            if before != circ {
                report.changed_display.push(addr);
            }
        }
        report
    }

    /// Breadth-first walk from the changed cells over reverse edges, recording every edge
    /// `source → dependent` so the topological pass can order them.
    fn collect_dirty(
        &self,
        changed: &[CellAddr],
    ) -> (HashSet<CellAddr>, HashMap<CellAddr, Vec<CellAddr>>) {
        let mut dirty: HashSet<CellAddr> = changed.iter().copied().collect();
        let mut successors: HashMap<CellAddr, Vec<CellAddr>> = HashMap::new();
        let mut queue: VecDeque<CellAddr> = changed.iter().copied().collect();
        while let Some(source) = queue.pop_front() {
            for dependent in self.graph.dependents_of(source) {
                successors.entry(source).or_default().push(dependent);
                if dirty.insert(dependent) {
                    queue.push_back(dependent);
                }
            }
        }
        (dirty, successors)
    }
}

/// Kahn's algorithm over the recorded edges. Returns the nodes that reached in-degree zero,
/// in dependency order. Nodes never returned are on a cycle or downstream of one.
fn topo_order(
    dirty: &HashSet<CellAddr>,
    successors: &HashMap<CellAddr, Vec<CellAddr>>,
) -> Vec<CellAddr> {
    let mut indegree: HashMap<CellAddr, usize> = dirty.iter().map(|a| (*a, 0)).collect();
    for targets in successors.values() {
        for t in targets {
            if let Some(d) = indegree.get_mut(t) {
                *d += 1;
            }
        }
    }
    let mut zero: Vec<CellAddr> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(a, _)| *a)
        .collect();
    zero.sort_unstable();
    let mut ready: VecDeque<CellAddr> = zero.into();
    let mut order = Vec::with_capacity(dirty.len());
    while let Some(node) = ready.pop_front() {
        order.push(node);
        if let Some(targets) = successors.get(&node) {
            for t in targets {
                if let Some(d) = indegree.get_mut(t) {
                    *d -= 1;
                    if *d == 0 {
                        ready.push_back(*t);
                    }
                }
            }
        }
    }
    order
}
