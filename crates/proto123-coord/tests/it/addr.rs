use proto123_coord::{CellAddr, CellRange, RefAddr, RefSyntax, letters_to_col};

#[test]
fn a1_round_trip_at_grid_corner() {
    let r = RefAddr::parse("XFD1048576").expect("corner parses");
    assert_eq!(r.addr, CellAddr::new(16_383, 1_048_575).unwrap());
    assert_eq!(r.to_string(), "XFD1048576");
}

#[test]
fn letters_are_case_insensitive_and_printed_upper() {
    assert_eq!(RefAddr::parse("b3").unwrap().to_string(), "B3");
}

#[test]
fn absolute_markers_are_kept_and_printed() {
    let r = RefAddr::parse("$A1").unwrap();
    assert!(r.col_abs && !r.row_abs);
    assert_eq!(r.to_string(), "$A1");
    assert_eq!(RefAddr::parse("A$1").unwrap().to_string(), "A$1");
    assert_eq!(RefAddr::parse("$A$1").unwrap().to_string(), "$A$1");
}

#[test]
fn out_of_grid_references_are_classified_not_parsed() {
    assert_eq!(RefAddr::classify("XFE1"), RefSyntax::OutOfGrid);
    assert_eq!(RefAddr::classify("A0"), RefSyntax::OutOfGrid);
    assert_eq!(RefAddr::classify("A1048577"), RefSyntax::OutOfGrid);
    assert_eq!(RefAddr::parse("XFE1"), None);
    assert_eq!(RefAddr::classify("ABCD1"), RefSyntax::NotARef);
    assert_eq!(RefAddr::classify("TOTAL"), RefSyntax::NotARef);
}

#[test]
fn shifting_keeps_absolute_parts_and_fails_off_grid() {
    let rel = RefAddr::parse("$A1").unwrap();
    let moved = rel.shifted(2, 3).unwrap();
    assert_eq!(moved.to_string(), "$A4");
    assert_eq!(RefAddr::parse("A1").unwrap().shifted(-1, 0), None);
    assert_eq!(RefAddr::parse("XFD1").unwrap().shifted(1, 0), None);
}

#[test]
fn column_letters_convert_both_ways() {
    assert_eq!(letters_to_col("A"), Some(0));
    assert_eq!(letters_to_col("Z"), Some(25));
    assert_eq!(letters_to_col("AA"), Some(26));
    assert_eq!(letters_to_col("XFD"), Some(16_383));
    assert_eq!(letters_to_col("XFE"), None);
}

#[test]
fn xfe_and_row_limits_are_rejected_by_constructor() {
    assert!(CellAddr::new(16_384, 0).is_none());
    assert!(CellAddr::new(0, 1_048_576).is_none());
}

#[test]
fn range_normalizes_from_any_corner() {
    let r = CellRange::parse("C5:A2").unwrap();
    assert_eq!(r.start(), CellAddr::new(0, 1).unwrap());
    assert_eq!(r.end(), CellAddr::new(2, 4).unwrap());
    assert_eq!(r.to_string(), "A2:C5");
}

#[test]
fn range_area_and_contains() {
    let r = CellRange::parse("A1:B3").unwrap();
    assert_eq!((r.width(), r.height(), r.area()), (2, 3, 6));
    assert!(r.contains(CellAddr::new(1, 2).unwrap()));
    assert!(!r.contains(CellAddr::new(2, 0).unwrap()));
}

#[test]
fn range_iterates_row_major() {
    let r = CellRange::parse("A1:B2").unwrap();
    let names: Vec<String> = r.iter().map(|a| a.to_string()).collect();
    assert_eq!(names, ["A1", "B1", "A2", "B2"]);
}

#[test]
fn cell_addr_orders_row_major() {
    let a1 = CellAddr::new(0, 0).unwrap();
    let b1 = CellAddr::new(1, 0).unwrap();
    let a2 = CellAddr::new(0, 1).unwrap();
    assert!(a1 < b1 && b1 < a2);
}
