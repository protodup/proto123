//! L4 engine: session state, interaction modes, the command registry, and the
//! command handlers. Every state change goes through [`Session::dispatch`].

mod announce;
mod command;
mod edit;
mod entry;
mod history;
pub mod menu;
pub mod menu_catalog;
mod nav;
mod point;
mod prompt;
mod session;
mod strings;

pub use command::{
    CaretKey, Command, CommandId, CommandSpec, HomeKind, PromptButton, REGISTRY, Then,
};
pub use entry::{EntryError, classify_entry};
pub use menu::MenuState;
pub use menu_catalog::{CATALOG, MenuKind, MenuNode};
pub use point::{PointState, is_point_eligible};
pub use prompt::{Prompt, RangePromptKind};
pub use session::{EditState, Mode, Outcome, Redraw, Session, Status, StatusKind, Viewport};
pub use strings::{StringKey, text};
