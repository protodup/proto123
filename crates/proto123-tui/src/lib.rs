//! L5 layout and input mapping, independent of the backend. Ratatui draws the frame; the
//! hosts forward raw events; gesture and hit-testing logic lives here.

mod input;
mod layout;
mod layout_map;
mod pixel;
mod raw;
mod render;
mod style;

pub use input::InputMapper;
pub use layout::{GridArea, grid_area, meets_minimum};
pub use layout_map::{HitTarget, LayoutMap};
pub use pixel::CellMetrics;
pub use raw::{Button, Key, Mods, Pos, RawEvent, RawKind};
pub use render::render;
