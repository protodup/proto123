//! L4 engine: session state, interaction modes, the command registry, and the
//! command handlers. Every state change goes through [`Session::dispatch`].

mod command;
mod edit;
mod entry;
mod nav;
mod session;
mod strings;

pub use command::{CaretKey, Command, CommandId, CommandSpec, HomeKind, REGISTRY, Then};
pub use entry::{EntryError, classify_entry};
pub use session::{EditState, Mode, Outcome, Redraw, Session, Status, StatusKind, Viewport};
pub use strings::{StringKey, text};
