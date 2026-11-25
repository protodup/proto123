//! Canvas-pixel metrics and pixel→cell mapping (web host). Pure math, unit-tested natively
//! (FR-031, FR-034, FR-043).

use proto123_theme::GridMetrics;

/// Cell geometry in CSS pixels, plus the device pixel ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellMetrics {
    pub cell_w: f64,
    pub cell_h: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub dpr: f64,
}

impl CellMetrics {
    /// Metrics from the theme's grid tokens (the values Ratzilla is expected to draw).
    pub fn from_theme(m: &GridMetrics, dpr: f64) -> Self {
        let origin = f64::from(m.origin_px);
        Self {
            cell_w: f64::from(m.cell_px.0),
            cell_h: f64::from(m.cell_px.1),
            origin_x: origin,
            origin_y: origin,
            dpr,
        }
    }

    /// Compares with the backend's reported cell size. On a mismatch the backend's size wins,
    /// so pointer targets stay aligned with what is drawn; the flag lets the engine warn
    /// (`diag.fontMetricsMismatch`).
    pub fn reconcile(&self, backend_cell_css: (f64, f64)) -> (Self, bool) {
        let (bw, bh) = backend_cell_css;
        if (bw - self.cell_w).abs() < f64::EPSILON && (bh - self.cell_h).abs() < f64::EPSILON {
            (*self, false)
        } else {
            (
                Self {
                    cell_w: bw,
                    cell_h: bh,
                    ..*self
                },
                true,
            )
        }
    }

    /// Visible grid size for a canvas of `css_w × css_h` CSS pixels.
    pub fn grid_size(&self, css_w: f64, css_h: f64) -> (u32, u32) {
        let cols = ((css_w - self.origin_x) / self.cell_w).floor().max(0.0);
        let rows = ((css_h - self.origin_y) / self.cell_h).floor().max(0.0);
        (cols as u32, rows as u32)
    }
}

/// Maps a CSS pixel coordinate `(x, y)` on the canvas to a TUI cell `(col, row)`, or `None`
/// when outside the `[0..cols, 0..rows)` character grid or inside the origin margin (FR-031).
pub fn pixel_to_cell(
    x: f64,
    y: f64,
    metrics: &CellMetrics,
    cols: u16,
    rows: u16,
) -> Option<(u16, u16)> {
    if !x.is_finite()
        || !y.is_finite()
        || !metrics.dpr.is_finite()
        || metrics.dpr <= 0.0
        || !metrics.cell_w.is_finite()
        || !metrics.cell_h.is_finite()
        || metrics.cell_w <= 0.0
        || metrics.cell_h <= 0.0
        || cols == 0
        || rows == 0
    {
        return None;
    }
    let lx = x - metrics.origin_x;
    let ly = y - metrics.origin_y;
    if lx < 0.0 || ly < 0.0 {
        return None;
    }
    let col = (lx / metrics.cell_w).floor();
    let row = (ly / metrics.cell_h).floor();
    if col < 0.0 || row < 0.0 {
        return None;
    }
    let col_u = col as u64;
    let row_u = row as u64;
    if col_u < u64::from(cols) && row_u < u64::from(rows) {
        Some((col_u as u16, row_u as u16))
    } else {
        None
    }
}
