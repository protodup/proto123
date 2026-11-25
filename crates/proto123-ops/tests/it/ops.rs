use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, CellRange, RefAddr};
use proto123_formula::Formula;
use proto123_model::{Alignment, CellContent};
use proto123_ops::Op;
use proto123_value::{ErrorKind, Value};

fn addr(s: &str) -> CellAddr {
    RefAddr::parse(s).unwrap().addr
}

fn range(s: &str) -> CellRange {
    CellRange::parse(s).unwrap()
}

fn num(n: f64) -> CellContent {
    CellContent::Number(n)
}

fn form(body: &str) -> CellContent {
    CellContent::Formula(Formula::parse(body).unwrap())
}

fn apply_op(calc: &mut CalcSheet, op: Op) {
    let delta = op.plan(calc);
    calc.apply(&delta.after_changes());
}

#[test]
fn erase_clears_range_and_recalculates_dependents() {
    let mut calc = CalcSheet::new();
    calc.apply(&[
        (addr("A1"), Some(num(10.0))),
        (addr("A2"), Some(num(20.0))),
        (addr("B1"), Some(form("SUM(A1:A2)"))),
    ]);
    assert_eq!(calc.value(addr("B1")), Value::Number(30.0));

    apply_op(
        &mut calc,
        Op::Erase {
            range: range("A1:A2"),
        },
    );
    assert_eq!(calc.value(addr("A1")), Value::Empty);
    assert_eq!(calc.value(addr("A2")), Value::Empty);
    assert_eq!(calc.value(addr("B1")), Value::Number(0.0));
}

#[test]
fn copy_tiles_single_cell_and_column_and_anchors_smaller_dest() {
    let mut calc = CalcSheet::new();
    for r in 1..=10 {
        calc.apply(&[(addr(&format!("A{r}")), Some(num(f64::from(r))))]);
    }
    calc.apply(&[(addr("B1"), Some(form("A1*2")))]);

    // Single cell copied into B1:B10 fills all ten; B2:B5 specifically gives =A2*2..=A5*2.
    apply_op(
        &mut calc,
        Op::Copy {
            src: range("B1:B1"),
            dest: range("B2:B5"),
        },
    );
    for r in 2..=5 {
        let c = calc.content(addr(&format!("B{r}"))).unwrap();
        assert_eq!(c.formula_bar_text(), format!("=A{r}*2"));
        assert_eq!(
            calc.value(addr(&format!("B{r}"))),
            Value::Number(f64::from(r * 2))
        );
    }

    // 1x3 source into B1:B10 fills B1:B9 (3 whole tiles), leaving B10 untouched.
    let mut c2 = CalcSheet::new();
    c2.apply(&[
        (addr("A1"), Some(num(1.0))),
        (addr("A2"), Some(num(2.0))),
        (addr("A3"), Some(num(3.0))),
        (addr("B10"), Some(num(99.0))),
    ]);
    apply_op(
        &mut c2,
        Op::Copy {
            src: range("A1:A3"),
            dest: range("B1:B10"),
        },
    );
    for r in 1..=9 {
        let expected = f64::from(((r - 1) % 3) + 1);
        assert_eq!(c2.value(addr(&format!("B{r}"))), Value::Number(expected));
    }
    assert_eq!(c2.value(addr("B10")), Value::Number(99.0));

    // Smaller destination (1x1) pastes source once anchored at top-left.
    let mut c3 = CalcSheet::new();
    c3.apply(&[
        (addr("A1"), Some(num(10.0))),
        (addr("A2"), Some(form("A1+5"))),
    ]);
    apply_op(
        &mut c3,
        Op::Copy {
            src: range("A1:A2"),
            dest: range("D1:D1"),
        },
    );
    assert_eq!(c3.value(addr("D1")), Value::Number(10.0));
    assert_eq!(c3.content(addr("D2")).unwrap().formula_bar_text(), "=D1+5");
    assert_eq!(c3.value(addr("D2")), Value::Number(15.0));
}

#[test]
fn copy_preserves_alignment_and_dollar_flags_and_handles_ranges_and_overlap() {
    let mut calc = CalcSheet::new();
    calc.apply(&[
        (addr("A1"), Some(num(3.0))),
        (addr("A2"), Some(num(4.0))),
        (addr("A3"), Some(form("SUM(A1:A2)"))),
        (
            addr("A4"),
            Some(CellContent::Label {
                text: "Tot".to_owned(),
                align: Alignment::Center,
                prefix: Some('^'),
            }),
        ),
        (addr("A5"), Some(form("$A$1+A2"))),
    ]);
    // Copy A3 right to B3 -> =SUM(B1:B2).
    apply_op(
        &mut calc,
        Op::Copy {
            src: range("A3:A3"),
            dest: range("B3:B3"),
        },
    );
    assert_eq!(
        calc.content(addr("B3")).unwrap().formula_bar_text(),
        "=SUM(B1:B2)"
    );

    // Copy A4..A5 to C4 -> preserves Center alignment and $A$1.
    apply_op(
        &mut calc,
        Op::Copy {
            src: range("A4:A5"),
            dest: range("C4:C4"),
        },
    );
    assert_eq!(calc.content(addr("C4")).unwrap().formula_bar_text(), "^Tot");
    assert_eq!(
        calc.content(addr("C5")).unwrap().formula_bar_text(),
        "=$A$1+C2"
    );

    // Overlapping copy A1:B1 -> B1:C1 matches capture-source-first.
    let mut ov = CalcSheet::new();
    ov.apply(&[(addr("A1"), Some(num(10.0))), (addr("B1"), Some(num(20.0)))]);
    apply_op(
        &mut ov,
        Op::Copy {
            src: range("A1:B1"),
            dest: range("B1:C1"),
        },
    );
    assert_eq!(ov.value(addr("A1")), Value::Number(10.0));
    assert_eq!(ov.value(addr("B1")), Value::Number(10.0));
    assert_eq!(ov.value(addr("C1")), Value::Number(20.0));
}

#[test]
fn move_relocates_redirects_and_marks_overwritten_refs_invalid() {
    let mut calc = CalcSheet::new();
    calc.apply(&[
        (addr("Z1"), Some(num(5.0))),
        (addr("A1"), Some(num(10.0))),
        (addr("A2"), Some(form("A1+Z1"))),
        (addr("C1"), Some(num(99.0))),
        // External single ref to A1 (redirect -> C1), external ref to overwritten C1 (-> #REF!),
        // wholly-inside range SUM(A1:A2) (-> SUM(C1:C2)), partially overlapping SUM(A1:B2) (unchanged).
        (addr("D1"), Some(form("A1"))),
        (addr("D2"), Some(form("C1+1"))),
        (addr("D3"), Some(form("SUM(A1:A2)"))),
        (addr("D4"), Some(form("SUM(A1:B2)"))),
    ]);

    apply_op(
        &mut calc,
        Op::Move {
            src: range("A1:A2"),
            dest: range("C1:C1"),
        },
    );

    assert_eq!(calc.value(addr("A1")), Value::Empty);
    assert_eq!(calc.value(addr("A2")), Value::Empty);
    assert_eq!(calc.value(addr("C1")), Value::Number(10.0));
    assert_eq!(
        calc.content(addr("C2")).unwrap().formula_bar_text(),
        "=C1+Z1"
    );
    assert_eq!(calc.value(addr("C2")), Value::Number(15.0));

    assert_eq!(calc.content(addr("D1")).unwrap().formula_bar_text(), "=C1");
    assert_eq!(calc.value(addr("D1")), Value::Number(10.0));

    assert_eq!(
        calc.content(addr("D2")).unwrap().formula_bar_text(),
        "=#REF!+1"
    );
    assert_eq!(calc.value(addr("D2")), Value::Error(ErrorKind::Ref));

    assert_eq!(
        calc.content(addr("D3")).unwrap().formula_bar_text(),
        "=SUM(C1:C2)"
    );
    assert_eq!(calc.value(addr("D3")), Value::Number(25.0));

    assert_eq!(
        calc.content(addr("D4")).unwrap().formula_bar_text(),
        "=SUM(A1:B2)"
    );
    assert_eq!(calc.value(addr("D4")), Value::Number(0.0));
}

#[test]
fn overlapping_move_matches_capture_source_first() {
    let mut calc = CalcSheet::new();
    calc.apply(&[
        (addr("A1"), Some(num(1.0))),
        (addr("A2"), Some(num(2.0))),
        (addr("A3"), Some(num(3.0))),
        (addr("B1"), Some(form("A1"))),
        (addr("B4"), Some(form("A4"))),
        (addr("A4"), Some(num(40.0))),
    ]);

    apply_op(
        &mut calc,
        Op::Move {
            src: range("A1:A3"),
            dest: range("A2:A2"),
        },
    );

    assert_eq!(calc.value(addr("A1")), Value::Empty);
    assert_eq!(calc.value(addr("A2")), Value::Number(1.0));
    assert_eq!(calc.value(addr("A3")), Value::Number(2.0));
    assert_eq!(calc.value(addr("A4")), Value::Number(3.0));
    assert_eq!(calc.content(addr("B1")).unwrap().formula_bar_text(), "=A2");
    assert_eq!(
        calc.content(addr("B4")).unwrap().formula_bar_text(),
        "=#REF!"
    );
}
