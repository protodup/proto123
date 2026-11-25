//! Navigation, selection, and viewport handlers (contracts/commands.md "Navigation & selection").

use crate::command::HomeKind;
use crate::session::Session;
use proto123_coord::{CellAddr, MAX_COL, MAX_ROW};

/// Moves `addr` by a signed offset, clamped to the grid (US1 sc.9).
pub(crate) fn offset_clamped(addr: CellAddr, dcol: i64, drow: i64) -> CellAddr {
    let col = i64::from(addr.col())
        .saturating_add(dcol)
        .clamp(0, i64::from(MAX_COL));
    let row = i64::from(addr.row())
        .saturating_add(drow)
        .clamp(0, i64::from(MAX_ROW));
    let col = u32::try_from(col).unwrap_or(0);
    let row = u32::try_from(row).unwrap_or(0);
    CellAddr::new(col, row).unwrap_or(addr)
}

impl Session {
    pub(crate) fn move_relative(&mut self, dcol: i64, drow: i64) {
        let next = offset_clamped(self.active(), dcol, drow);
        self.set_active(next);
    }

    pub(crate) fn set_active(&mut self, addr: CellAddr) {
        self.set_anchor(None);
        self.set_extend_mode(false);
        self.set_active_raw(addr);
        self.follow_addr(addr);
    }

    /// Extends selection from the current anchor (or current active cell) to `addr`.
    pub(crate) fn extend_selection(&mut self, addr: CellAddr) {
        if self.anchor().is_none() {
            self.set_anchor(Some(self.active()));
        }
        self.set_active_raw(addr);
        self.follow_addr(addr);
    }

    /// Toggles `[EXT]` (`F8`) mode in READY or POINT. Turning off preserves the selection/ref.
    pub(crate) fn toggle_extend_mode(&mut self) {
        match self.mode() {
            crate::session::Mode::Ready => {
                let next = !self.extend_mode();
                self.set_extend_mode(next);
                if next && self.anchor().is_none() {
                    self.set_anchor(Some(self.active()));
                }
            }
            crate::session::Mode::Point => {
                let next = !self.extend_mode();
                self.set_extend_mode(next);
                if next
                    && let Some(mut edit) = self.edit().cloned()
                    && let Some(mut pt) = edit.point.clone()
                {
                    pt.anchor = Some(pt.anchor.unwrap_or(pt.cursor));
                    edit.point = Some(pt);
                    self.set_edit(Some(edit));
                }
            }
            _ => {}
        }
    }

    pub(crate) fn page_scroll(&mut self, horizontal: bool, dir: i64) {
        let (cols, rows) = self.view_size();
        let step = if horizontal { cols } else { rows };
        let step = i64::from(step.max(1)).saturating_mul(dir);
        if horizontal {
            self.move_relative(step, 0);
        } else {
            self.move_relative(0, step);
        }
    }

    pub(crate) fn jump_home(&mut self, kind: HomeKind) {
        let current = self.active();
        let target = match kind {
            HomeKind::RowStart => CellAddr::new(0, current.row()).unwrap_or(current),
            HomeKind::A1 => CellAddr::ORIGIN,
        };
        self.set_active(target);
    }

    pub(crate) fn scroll_view(&mut self, dcol: i64, drow: i64) {
        let vp = self.viewport();
        let left = i64::from(vp.left)
            .saturating_add(dcol)
            .clamp(0, i64::from(MAX_COL));
        let top = i64::from(vp.top)
            .saturating_add(drow)
            .clamp(0, i64::from(MAX_ROW));
        self.set_viewport(
            u32::try_from(left).unwrap_or(0),
            u32::try_from(top).unwrap_or(0),
        );
    }

    /// Keeps the active cell inside the visible window (US1 sc.9, FR-033).
    pub(crate) fn follow_active(&mut self) {
        self.follow_addr(self.active());
    }

    /// Keeps `addr` inside the visible window.
    pub(crate) fn follow_addr(&mut self, addr: CellAddr) {
        let (cols, rows) = self.view_size();
        let cols = cols.max(1);
        let rows = rows.max(1);
        let vp = self.viewport();
        let col = addr.col();
        let row = addr.row();
        let left = if col < vp.left {
            col
        } else if col >= vp.left.saturating_add(cols) {
            col + 1 - cols
        } else {
            vp.left
        };
        let top = if row < vp.top {
            row
        } else if row >= vp.top.saturating_add(rows) {
            row + 1 - rows
        } else {
            vp.top
        };
        self.set_viewport(left, top);
    }
}
