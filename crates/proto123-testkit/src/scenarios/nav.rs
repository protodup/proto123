//! Navigation seed scenario (T035): arrows and Enter move the active cell and clamp at A1.

use crate::{Scenario, Snapshot};
use proto123_engine::Mode;
use std::collections::BTreeMap;

pub fn scenarios() -> Vec<Scenario> {
    vec![Scenario {
        name: "nav-clamps-at-a1",
        script: "<left><up><up>",
        expect: Snapshot {
            mode: Mode::Ready,
            active: "A1".to_owned(),
            cells: BTreeMap::new(),
        },
    }]
}
