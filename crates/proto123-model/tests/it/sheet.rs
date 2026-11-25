use proto123_coord::CellAddr;
use proto123_model::{CellContent, Sheet};

#[test]
fn writing_then_clearing_the_far_corner_leaves_no_entry() {
    let far = CellAddr::new(16_383, 1_048_575).unwrap();
    let mut sheet = Sheet::new();
    sheet.set_content(far, Some(CellContent::Number(1.0)));
    assert_eq!(sheet.populated(), 1);
    sheet.set_content(far, None);
    assert!(sheet.is_empty());
    assert_eq!(sheet.populated(), 0);
}
