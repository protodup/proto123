use crate::{Scenario, Snapshot};
use proto123_engine::Mode;
use std::collections::BTreeMap;

pub fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "us3_select_erase_undo_redo",
            script: "10<enter>20<enter>30<enter><up><up><up><f8><down><esc><del><c-z><c-y>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A2".to_owned(),
                cells: BTreeMap::from([("A3".to_owned(), "30".to_owned())]),
            },
        },
        Scenario {
            name: "us3_shift_arrow_erase_and_undo",
            script: "5<tab>15<enter><up><left><s-right><del><c-z>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "B1".to_owned(),
                cells: BTreeMap::from([
                    ("A1".to_owned(), "5".to_owned()),
                    ("B1".to_owned(), "15".to_owned()),
                ]),
            },
        },
    ]
}
