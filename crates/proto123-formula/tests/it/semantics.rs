use proto123_coord::{CellAddr, CellRange};
use proto123_formula::{CellSource, Formula, evaluate};
use proto123_value::Value;

struct Empty;

impl CellSource for Empty {
    fn value(&self, _: CellAddr) -> Value {
        Value::Empty
    }

    fn for_each_in_range(&self, _: CellRange, _: &mut dyn FnMut(CellAddr, &Value)) {}
}

fn shown(body: &str) -> String {
    evaluate(&Formula::parse(body).unwrap().expr, &Empty).grid_text(9)
}

#[test]
fn float_noise_is_hidden_at_grid_width() {
    assert_eq!(shown("0.1+0.2"), "0.3");
    assert_eq!(shown("1-0.9"), "0.1");
}

#[test]
fn unknown_function_keeps_its_canonical_text_for_editing() {
    let f = Formula::parse("summ(A1:A3)").unwrap();
    assert_eq!(f.canonical, "=SUMM(A1:A3)");
    assert_eq!(shown("SUMM(A1:A3)"), "#NAME?");
}

#[test]
fn booleans_take_part_in_arithmetic_as_one_and_zero() {
    assert_eq!(shown("TRUE+TRUE"), "2");
    assert_eq!(shown("FALSE*5"), "0");
}
