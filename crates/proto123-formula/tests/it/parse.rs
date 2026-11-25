use proto123_formula::{Expr, Formula, ParseErrorKind};

fn canon(body: &str) -> String {
    Formula::parse(body).expect("parses").canonical
}

#[test]
fn canonical_upper_cases_names_and_uses_commas() {
    assert_eq!(canon("sum(a1;a2)"), "=SUM(A1,A2)");
    assert_eq!(canon("sum(A1:b3)"), "=SUM(A1:B3)");
    assert_eq!(canon("$a$1"), "=$A$1");
}

#[test]
fn minimal_parentheses_keep_required_grouping() {
    assert_eq!(canon("(A1+B1)*C1"), "=(A1+B1)*C1");
    assert_eq!(canon("A1-(B1-C1)"), "=A1-(B1-C1)");
    assert_eq!(canon("(A1-B1)-C1"), "=A1-B1-C1");
    assert_eq!(canon("A1+(B1*C1)"), "=A1+B1*C1");
    assert_eq!(canon("2^(3^2)"), "=2^(3^2)");
    assert_eq!(canon("(2^3)^2"), "=2^3^2");
}

#[test]
fn round_trip_on_a_table_of_formulas() {
    let table = [
        "=SUM(A1:A2)",
        "=A1-(B1-C1)",
        "=-2^2",
        "=2^3^2",
        "=(A1+B1)*C1/2",
        "=AVERAGE(A1:B3,5,TRUE)",
        "=10/B1",
        "=SUMM(A1:A3)",
        "=#REF!+1",
        "=-(A1+B1)",
        "=A1--B1",
    ];
    for text in table {
        let first = Formula::parse(text.trim_start_matches('=')).expect("parses");
        let second = Formula::parse(first.canonical.trim_start_matches('=')).expect("reparses");
        assert_eq!(first.expr, second.expr, "round trip of {text}");
        assert_eq!(first.canonical, second.canonical, "stable canonical of {text}");
    }
}

#[test]
fn at_shorthand_is_accepted_for_functions_only() {
    assert_eq!(canon("@SUM(A1)"), "=SUM(A1)");
    assert!(Formula::parse("@A1").is_err());
}

#[test]
fn literals_and_bare_names() {
    assert_eq!(canon("true"), "=TRUE");
    assert!(matches!(Formula::parse("1.5E3").unwrap().expr, Expr::Number(n) if n == 1500.0));
    assert!(matches!(Formula::parse("TOTAL").unwrap().expr, Expr::ErrorLit(_)));
}

#[test]
fn out_of_grid_reference_is_ref_error_not_syntax_error() {
    let f = Formula::parse("XFE1+1").expect("parses");
    assert!(matches!(f.expr, Expr::Binary(_, _, _)));
    assert_eq!(canon("XFE1"), "=#REF!");
}

#[test]
fn unclosed_paren_reports_position_and_kind() {
    let err = Formula::parse("SUM(A1:").unwrap_err();
    assert_eq!(err.kind, ParseErrorKind::UnexpectedEnd);
    let err = Formula::parse("SUM(A1").unwrap_err();
    assert_eq!(err.kind, ParseErrorKind::UnclosedParen);
    assert_eq!(err.pos, 6);
}

#[test]
fn length_and_depth_limits_are_enforced() {
    let long = "1+".repeat(5_000) + "1";
    assert_eq!(Formula::parse(&long).unwrap_err().kind, ParseErrorKind::TooLong);
    let deep = "(".repeat(300) + "1" + &")".repeat(300);
    assert_eq!(Formula::parse(&deep).unwrap_err().kind, ParseErrorKind::TooDeep);
    let many_open = "(".repeat(100_000);
    assert!(Formula::parse(&many_open).is_err());
}
