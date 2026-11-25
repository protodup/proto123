use crate::common::{addr, session};
use proto123_engine::{Command, HomeKind, Redraw};

#[test]
fn arrows_move_and_clamp_at_the_grid_corners() {
    let mut s = session();
    s.dispatch(Command::NavMoveRelative { dcol: -1, drow: -1 });
    assert_eq!(s.active(), addr("A1"), "clamped at A1");
    s.dispatch(Command::NavSelectCell(addr("XFD1048576")));
    s.dispatch(Command::NavMoveRelative { dcol: 1, drow: 1 });
    assert_eq!(s.active(), addr("XFD1048576"), "clamped at XFD1048576");
}

#[test]
fn home_and_ctrl_home_jump() {
    let mut s = session();
    s.dispatch(Command::NavSelectCell(addr("D7")));
    s.dispatch(Command::NavJumpHome(HomeKind::RowStart));
    assert_eq!(s.active(), addr("A7"));
    s.dispatch(Command::NavJumpHome(HomeKind::A1));
    assert_eq!(s.active(), addr("A1"));
}

#[test]
fn page_scroll_moves_by_the_visible_size() {
    let mut s = session();
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s.dispatch(Command::NavPageScroll { horizontal: false, dir: 1 });
    assert_eq!(s.active(), addr("A21"));
    s.dispatch(Command::NavPageScroll { horizontal: true, dir: 1 });
    assert_eq!(s.active(), addr("I21"));
}

#[test]
fn viewport_follows_the_active_cell() {
    let mut s = session();
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s.dispatch(Command::NavSelectCell(addr("J30")));
    let vp = s.viewport();
    assert!(vp.left <= 9 && 9 < vp.left + 8, "column J visible");
    assert!(vp.top <= 29 && 29 < vp.top + 20, "row 30 visible");
}

#[test]
fn moving_to_the_same_cell_reports_no_redraw() {
    let mut s = session();
    let out = s.dispatch(Command::NavMoveRelative { dcol: -1, drow: 0 });
    assert_eq!(out.redraw, Redraw::None);
    let out = s.dispatch(Command::NavMoveRelative { dcol: 1, drow: 0 });
    assert_eq!(out.redraw, Redraw::Regions);
}

#[test]
fn resize_forces_a_full_redraw() {
    let mut s = session();
    let out = s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    assert_eq!(out.redraw, Redraw::Full);
}
