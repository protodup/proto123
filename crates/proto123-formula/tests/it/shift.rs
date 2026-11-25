use proto123_coord::RefAddr;
use proto123_formula::{Expr, Formula, RefRemap};
use proto123_value::ErrorKind;

fn ref_addr(s: &str) -> RefAddr {
    RefAddr::parse(s).unwrap()
}

#[test]
fn shifted_preserves_dollar_flags_and_turns_off_grid_into_ref_error() {
    let f = Formula::parse("A1+$B1+C$2+$D$3+SUM(A1:B2)").unwrap();
    let shifted = f.shifted(1, 2);
    assert_eq!(shifted.canonical, "=B3+$B3+D$2+$D$3+SUM(B3:C4)");

    // Shifting left from A1 leaves the grid -> #REF!.
    let left = f.shifted(-1, 0);
    assert_eq!(left.canonical, "=#REF!+$B1+B$2+$D$3+SUM(#REF!)");

    // Moving past XFD1048576 also becomes #REF!.
    let edge = Formula::parse("XFD1").unwrap().shifted(1, 0);
    assert_eq!(edge.expr, Expr::ErrorLit(ErrorKind::Ref));
    assert_eq!(edge.canonical, "=#REF!");
}

#[test]
fn remap_handles_keep_to_and_invalid() {
    let f = Formula::parse("A1+B2+SUM(C1:D2)").unwrap();
    let rewritten = f.remap(&|r| {
        if r.addr == ref_addr("A1").addr {
            RefRemap::To(ref_addr("Z9"))
        } else if r.addr == ref_addr("B2").addr {
            RefRemap::Invalid
        } else {
            RefRemap::Keep
        }
    });
    assert_eq!(rewritten.canonical, "=Z9+#REF!+SUM(C1:D2)");

    // Invalid endpoint inside a range invalidates the range to #REF!.
    let rng = f.remap(&|r| {
        if r.addr == ref_addr("D2").addr {
            RefRemap::Invalid
        } else {
            RefRemap::Keep
        }
    });
    assert_eq!(rng.canonical, "=A1+B2+SUM(#REF!)");
}
