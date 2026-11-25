//! `LayoutMap`: the rectangles drawn in the last frame, each tagged with what it is.
//! Mouse hit-testing reads this map, so drawing and targets come from one pass (FR-031).

use proto123_coord::CellAddr;
use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTarget {
    GridCell(CellAddr),
    ColHeader(u32),
    RowHeader(u32),
    FormulaBar { char_index: usize },
    AddressBox,
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutMap {
    entries: Vec<(Rect, HitTarget)>,
}

impl LayoutMap {
    pub fn push(&mut self, rect: Rect, target: HitTarget) {
        self.entries.push((rect, target));
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The target under a terminal cell. Later entries win where rectangles overlap.
    pub fn hit(&self, col: u16, row: u16) -> HitTarget {
        self.entries
            .iter()
            .rev()
            .find(|(r, _)| {
                col >= r.x && col < r.x.saturating_add(r.width) && row >= r.y && row < r.y.saturating_add(r.height)
            })
            .map_or(HitTarget::None, |(_, t)| *t)
    }

    pub fn entries(&self) -> &[(Rect, HitTarget)] {
        &self.entries
    }
}
