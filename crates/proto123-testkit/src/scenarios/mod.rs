//! The shared scenario suite. Each entry is replayed through every target's translation.

pub mod nav;
pub mod us1;
pub mod us2;
pub mod us3;
pub mod us4;
pub mod us5;

use crate::Scenario;

pub fn all() -> Vec<Scenario> {
    let mut v = Vec::new();
    v.extend(nav::scenarios());
    v.extend(us1::scenarios());
    v.extend(us2::scenarios());
    v.extend(us3::scenarios());
    v.extend(us4::scenarios());
    v.extend(us5::scenarios());
    v
}
