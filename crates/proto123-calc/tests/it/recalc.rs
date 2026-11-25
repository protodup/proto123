use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, RefAddr};
use proto123_formula::Formula;
use proto123_model::CellContent;
use proto123_value::{ErrorKind, Value};

fn at(text: &str) -> CellAddr {
    RefAddr::parse(text).unwrap().addr
}

fn num(n: f64) -> Option<CellContent> {
    Some(CellContent::Number(n))
}

fn formula(body: &str) -> Option<CellContent> {
    Some(CellContent::Formula(Formula::parse(body).unwrap()))
}

#[test]
fn sum_updates_when_an_input_changes() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A1"), num(120.0)), (at("A2"), num(80.0)), (at("A3"), formula("SUM(A1:A2)"))]);
    assert_eq!(calc.value(at("A3")), Value::Number(200.0));
    calc.apply(&[(at("A1"), num(150.0))]);
    assert_eq!(calc.value(at("A3")), Value::Number(230.0));
}

#[test]
fn evaluates_exactly_the_transitive_dependents_once_in_order() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A1"), num(1.0)), (at("B1"), formula("A1+1")), (at("C1"), formula("B1*2"))]);
    let report = calc.apply(&[(at("A1"), num(5.0))]);
    assert_eq!(report.evaluated, vec![at("B1"), at("C1")]);
    assert_eq!(calc.value(at("C1")), Value::Number(12.0));
}

#[test]
fn unrelated_cells_are_not_evaluated() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A1"), num(1.0)), (at("B1"), formula("A1+1")), (at("D9"), formula("7*6"))]);
    let report = calc.apply(&[(at("A1"), num(2.0))]);
    assert!(!report.evaluated.contains(&at("D9")));
    assert_eq!(report.evaluated, vec![at("B1")]);
}

#[test]
fn sc003_ten_dependents_of_ten_thousand_formulas() {
    let mut calc = CalcSheet::new();
    let mut batch = vec![(at("A1"), num(1.0))];
    for i in 0..10u32 {
        let addr = CellAddr::new(1, 100 + i).unwrap();
        batch.push((addr, formula("A1*2")));
    }
    for i in 0..9_990u32 {
        let addr = CellAddr::new(3, 100 + i).unwrap();
        batch.push((addr, formula("C1+1")));
    }
    calc.apply(&batch);
    let report = calc.apply(&[(at("A1"), num(9.0))]);
    assert_eq!(report.evaluated.len(), 10);
}

#[test]
fn self_reference_shows_circ() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A1"), formula("A1+1"))]);
    assert_eq!(calc.value(at("A1")), Value::Error(ErrorKind::Circ));
}

#[test]
fn two_cycle_with_dependent_and_recovery() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A1"), formula("B1")), (at("B1"), formula("A1")), (at("C1"), formula("A1+1"))]);
    assert_eq!(calc.value(at("A1")), Value::Error(ErrorKind::Circ));
    assert_eq!(calc.value(at("B1")), Value::Error(ErrorKind::Circ));
    assert_eq!(calc.value(at("C1")), Value::Error(ErrorKind::Circ));
    calc.apply(&[(at("B1"), num(4.0))]);
    assert_eq!(calc.value(at("A1")), Value::Number(4.0));
    assert_eq!(calc.value(at("C1")), Value::Number(5.0));
}

#[test]
fn erasing_an_input_propagates_as_zero() {
    let mut calc = CalcSheet::new();
    calc.apply(&[(at("A1"), num(3.0)), (at("B1"), formula("A1*2"))]);
    calc.apply(&[(at("A1"), None)]);
    assert_eq!(calc.value(at("B1")), Value::Number(0.0));
}
