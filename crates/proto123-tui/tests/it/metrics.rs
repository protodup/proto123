use proto123_theme::{GridMetrics, high_contrast};
use proto123_tui::CellMetrics;

#[test]
fn from_theme_copies_cell_and_origin_and_keeps_dpr() {
    let m = CellMetrics::from_theme(&high_contrast().metrics, 2.0);
    assert_eq!((m.cell_w, m.cell_h), (10.0, 19.0));
    assert_eq!((m.origin_x, m.origin_y), (5.0, 5.0));
    assert_eq!(m.dpr, 2.0);
}

#[test]
fn reconcile_keeps_theme_values_when_backend_agrees() {
    let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
    let (used, mismatch) = m.reconcile((10.0, 19.0));
    assert_eq!(used, m);
    assert!(!mismatch);
}

#[test]
fn reconcile_uses_backend_values_on_mismatch() {
    let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
    let (used, mismatch) = m.reconcile((9.0, 18.0));
    assert!(mismatch);
    assert_eq!((used.cell_w, used.cell_h), (9.0, 18.0));
}

#[test]
fn grid_size_from_canvas_pixels() {
    let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
    assert_eq!(m.grid_size(800.0, 480.0), (79, 25));
}
