//! `LayoutMap`: the rectangles drawn in the last frame, each tagged with what it is.
//! Mouse hit-testing reads this map, so drawing and targets come from one pass (FR-031).

use proto123_coord::CellAddr;
use proto123_engine::PromptButton;
use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeDir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTarget {
    GridCell(CellAddr),
    ColHeader(u32),
    RowHeader(u32),
    FormulaBar { char_index: usize },
    AddressBox,
    MenuItem { level: usize, index: usize },
    PromptField { char_index: usize },
    PromptButton(PromptButton),
    GridEdge(EdgeDir),
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutMap {
    entries: Vec<(Rect, HitTarget)>,
    grid_rect: Option<Rect>,
}

impl LayoutMap {
    pub fn push(&mut self, rect: Rect, target: HitTarget) {
        self.entries.push((rect, target));
    }

    pub fn set_grid_rect(&mut self, rect: Rect) {
        self.grid_rect = Some(rect);
    }

    pub fn grid_rect(&self) -> Option<Rect> {
        self.grid_rect
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the edge direction when `(col, row)` is outside the drawn grid cell rectangle
    /// during a drag (used for edge auto-scroll, US4 sc.3).
    pub fn edge_for(&self, col: u16, row: u16) -> Option<EdgeDir> {
        let rect = self.grid_rect?;
        if row < rect.y {
            Some(EdgeDir::Up)
        } else if row >= rect.bottom() {
            Some(EdgeDir::Down)
        } else if col < rect.x {
            Some(EdgeDir::Left)
        } else if col >= rect.right() {
            Some(EdgeDir::Right)
        } else {
            None
        }
    }

    /// Finds the closest `GridCell` inside `grid_rect` for a pointer position `(col, row)`.
    pub fn nearest_grid_cell(&self, col: u16, row: u16) -> Option<CellAddr> {
        let rect = self.grid_rect?;
        if rect.width == 0 || rect.height == 0 {
            return None;
        }
        let cx = col.clamp(rect.x, rect.right().saturating_sub(1));
        let cy = row.clamp(rect.y, rect.bottom().saturating_sub(1));
        match self.hit(cx, cy) {
            HitTarget::GridCell(addr) => Some(addr),
            _ => None,
        }
    }

    /// The target under a terminal cell. Later entries win where rectangles overlap.
    /// Text-field targets compute `char_index` from the column offset inside the rect.
    pub fn hit(&self, col: u16, row: u16) -> HitTarget {
        self.entries
            .iter()
            .rev()
            .find(|(r, _)| {
                col >= r.x
                    && col < r.x.saturating_add(r.width)
                    && row >= r.y
                    && row < r.y.saturating_add(r.height)
            })
            .map_or(HitTarget::None, |(r, t)| match *t {
                HitTarget::FormulaBar { .. } => HitTarget::FormulaBar {
                    char_index: usize::from(col.saturating_sub(r.x)),
                },
                HitTarget::PromptField { .. } => HitTarget::PromptField {
                    char_index: usize::from(col.saturating_sub(r.x)),
                },
                other => other,
            })
    }

    pub fn entries(&self) -> &[(Rect, HitTarget)] {
        &self.entries
    }
}
