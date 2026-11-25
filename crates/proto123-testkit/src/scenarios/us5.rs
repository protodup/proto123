use crate::{Scenario, Snapshot};
use proto123_engine::Mode;
use std::collections::BTreeMap;

pub fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "us5_slash_menu_range_erase",
            script: "10<enter>20<enter><up><up><f8><down><esc>/re",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A2".to_owned(),
                cells: BTreeMap::new(),
            },
        },
        Scenario {
            name: "us5_file_new_confirm_clears_workbook",
            script: "42<enter>/fn<tab><enter>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A1".to_owned(),
                cells: BTreeMap::new(),
            },
        },
    ]
}
