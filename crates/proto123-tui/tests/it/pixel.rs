use proto123_theme::GridMetrics;
use proto123_tui::{CellMetrics, pixel_to_cell};

#[test]
fn pixel_to_cell_handles_dpr_edges_origin_and_out_of_bounds() {
    for dpr in [1.0, 1.25, 2.0] {
        let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, dpr);
        // Inside origin margin -> None.
        assert_eq!(pixel_to_cell(4.9, 5.0, &m, 80, 24), None);
        assert_eq!(pixel_to_cell(5.0, 4.9, &m, 80, 24), None);
        assert_eq!(pixel_to_cell(-1.0, 10.0, &m, 80, 24), None);
        assert_eq!(pixel_to_cell(10.0, -0.1, &m, 80, 24), None);

        // Exact top-left of cell (0, 0).
        assert_eq!(pixel_to_cell(5.0, 5.0, &m, 80, 24), Some((0, 0)));
        // Last pixel inside cell (0, 0) (cell is 10x19).
        assert_eq!(pixel_to_cell(14.99, 23.99, &m, 80, 24), Some((0, 0)));
        // Exact boundary into cell (1, 1).
        assert_eq!(pixel_to_cell(15.0, 24.0, &m, 80, 24), Some((1, 1)));
        // Last valid cell (79, 23).
        assert_eq!(
            pixel_to_cell(5.0 + 79.0 * 10.0, 5.0 + 23.0 * 19.0, &m, 80, 24),
            Some((79, 23))
        );
        // Just past the last column or row -> None.
        assert_eq!(pixel_to_cell(5.0 + 80.0 * 10.0, 5.0, &m, 80, 24), None);
        assert_eq!(pixel_to_cell(5.0, 5.0 + 24.0 * 19.0, &m, 80, 24), None);
    }
}

#[test]
fn non_finite_or_zero_metrics_return_none() {
    let m = CellMetrics::from_theme(&GridMetrics::DEFAULT, 1.0);
    assert_eq!(pixel_to_cell(f64::NAN, 10.0, &m, 80, 24), None);
    assert_eq!(pixel_to_cell(10.0, f64::INFINITY, &m, 80, 24), None);
    let zero_dpr = CellMetrics { dpr: 0.0, ..m };
    assert_eq!(pixel_to_cell(10.0, 10.0, &zero_dpr, 80, 24), None);
}
