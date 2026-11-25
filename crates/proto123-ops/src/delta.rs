use proto123_coord::CellAddr;
use proto123_model::CellContent;

/// Minimal reversible mutation of a set of cells: `(addr, before, after)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CellDelta {
    pub changes: Vec<(CellAddr, Option<CellContent>, Option<CellContent>)>,
}

impl CellDelta {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Swaps `before` and `after` for every cell so undo becomes redo and vice versa.
    pub fn invert(&self) -> Self {
        let changes = self
            .changes
            .iter()
            .map(|(addr, before, after)| (*addr, after.clone(), before.clone()))
            .collect();
        Self { changes }
    }

    /// Changes to pass to `CalcSheet::apply` when applying (doing or redoing) this delta.
    pub fn after_changes(&self) -> Vec<(CellAddr, Option<CellContent>)> {
        self.changes
            .iter()
            .map(|(addr, _, after)| (*addr, after.clone()))
            .collect()
    }

    /// Changes to pass to `CalcSheet::apply` when undoing this delta.
    pub fn before_changes(&self) -> Vec<(CellAddr, Option<CellContent>)> {
        self.changes
            .iter()
            .map(|(addr, before, _)| (*addr, before.clone()))
            .collect()
    }
}
