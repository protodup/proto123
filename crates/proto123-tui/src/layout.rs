//! Screen geometry: the same numbers drive drawing, the visible grid size, and hit-testing.

use proto123_theme::GridMetrics;

/// Rows above the grid: control line, root-bar line, column headers.
pub(crate) const HEADER_ROWS: u16 = 3;
/// Width of the address box on the control line (`[A1]` padded).
pub(crate) const ADDRESS_BOX_WIDTH: u16 = 8;
/// Rows below the grid: status bar.
pub(crate) const FOOTER_ROWS: u16 = 1;

/// Visible grid size in cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridArea {
    pub cols: u32,
    pub rows: u32,
}

/// True when the terminal or canvas is at least the minimum (FR-033, 80×24).
pub fn meets_minimum(cols: u16, rows: u16, metrics: &GridMetrics) -> bool {
    cols >= metrics.min_cols && rows >= metrics.min_rows
}

/// How many grid cells fit in a `cols × rows` area.
pub fn grid_area(cols: u16, rows: u16, metrics: &GridMetrics) -> GridArea {
    let width = cols.saturating_sub(metrics.row_header_width);
    let height = rows.saturating_sub(HEADER_ROWS + FOOTER_ROWS);
    GridArea {
        cols: u32::from(width / metrics.col_width.max(1)),
        rows: u32::from(height),
    }
}
