use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, RefAddr};
use proto123_formula::Formula;
use proto123_model::CellContent;
use proto123_value::{ErrorKind, Value};

fn at(text: &str) -> CellAddr {
    RefAddr::parse(text).unwrap().addr
}

fn row(r: u32) -> CellAddr {
    CellAddr::new(0, r).unwrap()
}

fn formula(body: &str) -> Option<CellContent> {
    Some(CellContent::Formula(Formula::parse(body).unwrap()))
}

/// A ring of `n` formulas: A1 = A_n + 1 and A_i = A_(i-1) + 1 for the rest.
fn ring(n: u32) -> Vec<(CellAddr, Option<CellContent>)> {
    (0..n)
        .map(|i| {
            let prev = if i == 0 { n - 1 } else { i - 1 };
            (row(i), formula(&format!("A{}+1", prev + 1)))
        })
        .collect()
}

#[test]
fn five_hundred_cycle_is_all_circ() {
    let mut calc = CalcSheet::new();
    calc.apply(&ring(500));
    assert_eq!(calc.value(row(0)), Value::Error(ErrorKind::Circ));
    assert_eq!(calc.value(row(499)), Value::Error(ErrorKind::Circ));
}

#[test]
fn ten_thousand_cycle_is_all_circ_without_recursion() {
    let mut calc = CalcSheet::new();
    calc.apply(&ring(10_000));
    assert_eq!(calc.value(row(9_999)), Value::Error(ErrorKind::Circ));
}

#[test]
fn range_that_includes_its_own_cell_is_a_cycle() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A5"), formula("SUM(A1:A10)"))]);
    assert_eq!(calc.value(at("A5")), Value::Error(ErrorKind::Circ));
}

#[test]
fn ten_thousand_deep_chain_recalculates_correctly() {
    let mut calc = CalcSheet::new();
    let mut batch = vec![(row(0), Some(CellContent::Number(1.0)))];
    for r in 1..10_000u32 {
        batch.push((row(r), formula(&format!("A{}+1", r))));
    }
    calc.apply(&batch);
    assert_eq!(calc.value(row(9_999)), Value::Number(10_000.0));
    calc.apply(&[(row(0), Some(CellContent::Number(5.0)))]);
    assert_eq!(calc.value(row(9_999)), Value::Number(10_004.0));
}

#[test]
fn whole_grid_sum_reads_only_the_populated_cells() {
    let mut calc = CalcSheet::new();
    let mut batch: Vec<_> = (0..10u32).map(|r| (row(r), Some(CellContent::Number(f64::from(r + 1))))).collect();
    // The formula sits in the last row, outside the range it sums (so it is not a cycle).
    batch.push((at("C1048576"), formula("SUM(A1:XFD1048575)")));
    calc.apply(&batch);
    assert_eq!(calc.value(at("C1048576")), Value::Number(55.0));
}
