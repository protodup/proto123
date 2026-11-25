//! US1 Independent Test: a keyboard-only small calculating model.

use crate::{Scenario, Snapshot};
use proto123_engine::Mode;
use std::collections::BTreeMap;

fn cells(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(a, v)| ((*a).to_owned(), (*v).to_owned()))
        .collect()
}

pub fn scenarios() -> Vec<Scenario> {
    vec![Scenario {
        name: "us1-sum-model-recalculates",
        // 120⏎ 80⏎ → A3; Total in B3 (Left commits and moves to A3); =SUM(A1:A2) → 200;
        // up to A1, 150⏎ → A3 becomes 230.
        script: "120<enter>80<enter><right>Total<left>=SUM(A1:A2)<enter><up><up><up>150<enter>",
        expect: Snapshot {
            mode: Mode::Ready,
            active: "A2".to_owned(),
            cells: cells(&[("A1", "150"), ("A2", "80"), ("A3", "230"), ("B3", "Total")]),
        },
    }]
}
