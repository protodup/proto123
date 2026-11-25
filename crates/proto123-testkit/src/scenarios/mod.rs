//! The shared scenario suite. Each entry is replayed through every target's translation.

pub mod nav;
pub mod us1;

use crate::Scenario;

pub fn all() -> Vec<Scenario> {
    let mut v = Vec::new();
    v.extend(nav::scenarios());
    v.extend(us1::scenarios());
    v
}
