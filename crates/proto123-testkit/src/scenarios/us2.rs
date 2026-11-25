//! US2 scenarios: error tokens that spread and recover, cycles, and syntax diagnostics.

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
    vec![
        Scenario {
            name: "us2-division-by-empty-shows-div0",
            script: "=10/B1<enter>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A2".to_owned(),
                cells: cells(&[("A1", "#DIV/0!")]),
            },
        },
        Scenario {
            name: "us2-unknown-function-shows-name",
            script: "=SUMM(1)<enter>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A2".to_owned(),
                cells: cells(&[("A1", "#NAME?")]),
            },
        },
        Scenario {
            name: "us2-text-in-arithmetic-shows-value",
            script: "'abc<enter>=A1+1<enter>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "A3".to_owned(),
                cells: cells(&[("A1", "abc"), ("A2", "#VALUE!")]),
            },
        },
        Scenario {
            // A1 = B1 and A2 = A1 form a cycle; typing a constant into B1 breaks it and both recover.
            name: "us2-cycle-recovers-when-broken",
            script: "=B1<enter>=A1<enter><up><up><right>7<enter>",
            expect: Snapshot {
                mode: Mode::Ready,
                active: "B2".to_owned(),
                cells: cells(&[("A1", "7"), ("A2", "7"), ("B1", "7")]),
            },
        },
        Scenario {
            // The unparsable entry keeps the editor open (VALUE) and stores nothing.
            name: "us2-syntax-error-keeps-editing",
            script: "=SUM(<enter>",
            expect: Snapshot {
                mode: Mode::Value,
                active: "A1".to_owned(),
                cells: BTreeMap::new(),
            },
        },
    ]
}
