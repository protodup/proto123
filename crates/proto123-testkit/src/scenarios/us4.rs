use crate::{Scenario, Snapshot};
use proto123_engine::Mode;
use std::collections::BTreeMap;

/// US4 pointer scenarios in cell coordinates. Both targets translate them through their
/// native or DOM/pixel paths and assert identical snapshots.
pub fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "us4_click_double_click_edit_and_wheel",
            // Click B2 (col=14, row=3+1=4), double-click B2 -> EDIT, type 42, Enter, wheel down.
            script: "10<enter><pdown:14,4><pup:14,4><pdown:14,4><pup:14,4>42<enter><wheel:0,1>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "B3".to_owned(),
                cells: BTreeMap::from([
                    ("A1".to_owned(), "10".to_owned()),
                    ("B2".to_owned(), "42".to_owned()),
                ]),
            },
        },
        Scenario {
            name: "us4_point_by_click_and_drag",
            // Type 5 in A1, 7 in A2, click C1 (col=23, row=3), type =SUM(, drag A1..A2, type ), Enter.
            script: "5<enter>7<enter><pdown:23,3><pup:23,3>=SUM(<pdown:5,3><pmove:5,4><pup:5,4>)<enter>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "C2".to_owned(),
                cells: BTreeMap::from([
                    ("A1".to_owned(), "5".to_owned()),
                    ("A2".to_owned(), "7".to_owned()),
                    ("C1".to_owned(), "12".to_owned()),
                ]),
            },
        },
        Scenario {
            name: "us4_click_edit_menu_then_undo",
            // Enter 99 in A1, click Edit in root bar (x=8, y=1), click Undo (x=1, y=1).
            script: "99<enter><pdown:8,1><pup:8,1><pdown:1,1><pup:1,1>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A2".to_owned(),
                cells: BTreeMap::new(),
            },
        },
    ]
}
