use proto123_coord::{CellAddr, CellRange};
use proto123_formula::{CellSource, Formula, evaluate};
use proto123_value::{ErrorKind, Value};
use std::collections::BTreeMap;

#[derive(Default)]
struct Sheet(BTreeMap<CellAddr, Value>);

impl Sheet {
    fn with(mut self, addr: &str, v: Value) -> Self {
        self.0.insert(proto123_coord::RefAddr::parse(addr).unwrap().addr, v);
        self
    }
}

impl CellSource for Sheet {
    fn value(&self, addr: CellAddr) -> Value {
        self.0.get(&addr).cloned().unwrap_or(Value::Empty)
    }

    fn for_each_in_range(&self, range: CellRange, f: &mut dyn FnMut(CellAddr, &Value)) {
        for (addr, v) in &self.0 {
            if range.contains(*addr) {
                f(*addr, v);
            }
        }
    }
}

fn run(body: &str, sheet: &Sheet) -> Value {
    evaluate(&Formula::parse(body).unwrap().expr, sheet)
}

fn num(n: f64) -> Value {
    Value::Number(n)
}

#[test]
fn arithmetic_operators_and_precedence() {
    let s = Sheet::default();
    assert_eq!(run("2+3*4", &s), num(14.0));
    assert_eq!(run("(2+3)*4", &s), num(20.0));
    assert_eq!(run("-2^2", &s), num(4.0));
    assert_eq!(run("2^3^2", &s), num(64.0));
    assert_eq!(run("10/4", &s), num(2.5));
    assert_eq!(run("+5", &s), num(5.0));
}

#[test]
fn empty_and_bool_operands_coerce() {
    let s = Sheet::default().with("A1", Value::Bool(true));
    assert_eq!(run("A1+B1", &s), num(1.0));
    assert_eq!(run("TRUE*4", &s), num(4.0));
    assert_eq!(run("C1", &s), num(0.0));
}

#[test]
fn aggregates_over_ranges_skip_text_empty_and_bool() {
    let s = Sheet::default()
        .with("A1", num(120.0))
        .with("A2", num(80.0))
        .with("A3", Value::Text("note".into()))
        .with("A4", Value::Bool(true));
    assert_eq!(run("SUM(A1:A4)", &s), num(200.0));
    assert_eq!(run("COUNT(A1:A4)", &s), num(2.0));
    assert_eq!(run("AVERAGE(A1:A2)", &s), num(100.0));
    assert_eq!(run("MIN(A1:A2)", &s), num(80.0));
    assert_eq!(run("MAX(A1:A2,500)", &s), num(500.0));
}

#[test]
fn aggregates_mix_direct_args_and_ranges() {
    let s = Sheet::default().with("A1", num(1.0));
    assert_eq!(run("SUM(A1,2,TRUE)", &s), num(4.0));
}

#[test]
fn direct_text_argument_is_value_error() {
    let s = Sheet::default().with("A1", Value::Text("x".into()));
    assert_eq!(run("SUM(A1)", &s), Value::Error(ErrorKind::Value));
    // Inside a range the same text is skipped, not an error.
    assert_eq!(run("SUM(A1:A1)", &s), num(0.0));
}

#[test]
fn unknown_function_is_name_error() {
    let s = Sheet::default();
    assert_eq!(run("SUMM(1)", &s), Value::Error(ErrorKind::Name));
}

#[test]
fn formula_reading_empty_is_zero_at_top_level() {
    let s = Sheet::default();
    assert_eq!(run("A1", &s), num(0.0));
}

#[test]
fn references_list_cells_and_ranges() {
    let f = Formula::parse("SUM(A1:A2)+B3").unwrap();
    let (cells, ranges) = f.references();
    assert_eq!(cells.len(), 1);
    assert_eq!(ranges, vec![CellRange::parse("A1:A2").unwrap()]);
}

#[test]
fn division_by_an_empty_cell_is_div0() {
    let s = Sheet::default().with("B1", Value::Empty);
    assert_eq!(run("10/B1", &s), Value::Error(ErrorKind::Div0));
}

#[test]
fn arithmetic_on_text_is_value_error() {
    let s = Sheet::default().with("A1", Value::Text("abc".into()));
    assert_eq!(run("A1+1", &s), Value::Error(ErrorKind::Value));
}

#[test]
fn first_error_left_to_right_wins() {
    let s = Sheet::default();
    assert_eq!(run("#REF!+#DIV/0!", &s), Value::Error(ErrorKind::Ref));
    assert_eq!(run("#NAME?*A1+#VALUE!", &s), Value::Error(ErrorKind::Name));
}

#[test]
fn average_and_aggregates_over_nothing() {
    let s = Sheet::default();
    assert_eq!(run("AVERAGE(A1:A3)", &s), Value::Error(ErrorKind::Div0));
    assert_eq!(run("SUM(A1:A3)", &s), num(0.0));
    assert_eq!(run("MIN(A1:A3)", &s), num(0.0));
    assert_eq!(run("COUNT(A1:A3)", &s), num(0.0));
}

#[test]
fn overflow_to_infinity_is_value_error() {
    let s = Sheet::default();
    assert_eq!(run("10^400", &s), Value::Error(ErrorKind::Value));
}

#[test]
fn error_inside_a_range_spreads_to_the_aggregate() {
    let s = Sheet::default().with("A1", num(1.0)).with("A2", Value::Error(ErrorKind::Div0));
    assert_eq!(run("SUM(A1:A2)", &s), Value::Error(ErrorKind::Div0));
}
