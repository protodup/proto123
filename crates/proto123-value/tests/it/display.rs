use proto123_value::{ErrorKind, Value, format_full, format_general};

fn general(n: f64) -> String {
    format_general(n, 9)
}

#[test]
fn fr006_table_at_width_nine() {
    assert_eq!(general(0.1 + 0.2), "0.3");
    assert_eq!(general(1.0 - 0.9), "0.1");
    assert_eq!(general(10.0 / 4.0), "2.5");
    assert_eq!(general(1.0 / 3.0), "0.3333333");
    assert_eq!(general(0.000_001_234), "0.0000012");
    assert_eq!(general(123_456_789_012.0), "1.235E+11");
    assert_eq!(general(0.000_000_001_2), "1.2E-09");
}

#[test]
fn negative_zero_shows_zero() {
    assert_eq!(general(-0.0), "0");
    assert_eq!(format_full(-0.0), "0");
}

#[test]
fn ties_round_half_away_from_zero() {
    assert_eq!(format_general(2.5, 1), "3");
    assert_eq!(format_general(-2.5, 2), "-3");
    assert_eq!(format_general(0.125, 4), "0.13");
}

#[test]
fn hash_fill_when_width_below_scientific_minimum() {
    assert_eq!(format_general(123_456.0, 3), "###");
}

#[test]
fn full_precision_is_at_most_fifteen_significant_digits() {
    assert_eq!(format_full(0.1 + 0.2), "0.3");
    assert_eq!(format_full(1.0 / 3.0), "0.333333333333333");
    assert_eq!(format_full(1234.5), "1234.5");
}

#[test]
fn error_tokens_and_booleans() {
    assert_eq!(ErrorKind::Div0.token(), "#DIV/0!");
    assert_eq!(ErrorKind::Ref.token(), "#REF!");
    assert_eq!(ErrorKind::Name.token(), "#NAME?");
    assert_eq!(ErrorKind::Value.token(), "#VALUE!");
    assert_eq!(ErrorKind::Circ.token(), "#CIRC!");
    assert_eq!(Value::Bool(true).grid_text(9), "TRUE");
    assert_eq!(Value::Bool(false).grid_text(9), "FALSE");
}

#[test]
fn non_finite_numbers_become_value_errors() {
    assert_eq!(Value::number(f64::INFINITY), Value::Error(ErrorKind::Value));
    assert_eq!(Value::number(f64::NAN), Value::Error(ErrorKind::Value));
}
