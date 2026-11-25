use crate::delta::CellDelta;
use std::collections::VecDeque;

/// Default undo stack depth (SC-006 requires at least 100 steps).
pub const DEFAULT_CAPACITY: usize = 1_000;

/// Bounded undo/redo history of reversible [`CellDelta`]s.
#[derive(Clone, Debug)]
pub struct UndoStack {
    undo: VecDeque<CellDelta>,
    redo: Vec<CellDelta>,
    capacity: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoStack {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            capacity,
        }
    }

    /// Records a new mutating step and discards any redo history (FR-018).
    /// Empty deltas are ignored so no-op commands create no undo step.
    pub fn push(&mut self, delta: CellDelta) {
        if delta.is_empty() {
            return;
        }
        self.redo.clear();
        if self.capacity == 0 {
            return;
        }
        if self.undo.len() == self.capacity {
            self.undo.pop_front();
        }
        self.undo.push_back(delta);
    }

    /// Pops the most recent delta for undoing and moves it to the redo stack.
    pub fn undo(&mut self) -> Option<CellDelta> {
        let delta = self.undo.pop_back()?;
        self.redo.push(delta.clone());
        Some(delta)
    }

    /// Pops the most recently undone delta for redoing and moves it back to the undo stack.
    pub fn redo(&mut self) -> Option<CellDelta> {
        let delta = self.redo.pop()?;
        if self.capacity > 0 {
            if self.undo.len() == self.capacity {
                self.undo.pop_front();
            }
            self.undo.push_back(delta.clone());
        }
        Some(delta)
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }
}
