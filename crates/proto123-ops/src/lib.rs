//! L2 reversible operations: atomic Erase/Copy/Move/SetCell deltas and the Undo/Redo stack.

mod delta;
mod ops;
mod undo;

pub use delta::CellDelta;
pub use ops::Op;
pub use undo::{DEFAULT_CAPACITY, UndoStack};
