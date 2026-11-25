use proto123_coord::{CellAddr, RefAddr};
use proto123_engine::{Command, Session, Then};
use proto123_theme::high_contrast;
use proto123_tui::render;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

const AREA: Rect = Rect { x: 0, y: 0, width: 80, height: 24 };
// Layout constants of the 80×24 frame: control line y=0, column headers y=2, first grid row y=3,
// status bar y=23; row header 5 wide, columns 9 wide.
const CONTROL_Y: u16 = 0;
const HEADER_Y: u16 = 2;
const ROW1_Y: u16 = 3;
const ROW3_Y: u16 = 5;
const STATUS_Y: u16 = 23;

fn frame(s: &Session) -> Buffer {
    let mut buf = Buffer::empty(AREA);
    render(s, AREA, &mut buf);
    buf
}

fn changed(before: &Buffer, after: &Buffer) -> Vec<(u16, u16)> {
    before.diff(after).into_iter().map(|(x, y, _)| (x, y)).collect()
}

fn addr(s: &str) -> CellAddr {
    RefAddr::parse(s).unwrap().addr
}

fn ready_session() -> Session {
    let mut s = Session::new(high_contrast());
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s
}

#[test]
fn committing_a_precedent_changes_only_the_control_line_and_the_affected_cells() {
    let mut s = ready_session();
    s.dispatch(Command::CellBeginEntry('1'));
    s.dispatch(Command::CellCommitEdit(Then::Move { dcol: 0, drow: 1 }));
    s.dispatch(Command::NavSelectCell(addr("A3")));
    for c in "SUM(A1:A2)".chars() {
        if c == 'S' {
            s.dispatch(Command::CellBeginEntry('='));
        }
        s.dispatch(Command::CellEditInput(c));
    }
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::NavSelectCell(addr("A1")));
    s.dispatch(Command::CellBeginEdit);
    s.dispatch(Command::CellEditCaret(proto123_engine::CaretKey::End));
    let before = frame(&s);
    s.dispatch(Command::CellEditInput('5'));
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    let after = frame(&s);

    let cells = changed(&before, &after);
    assert!(!cells.is_empty());
    for (x, y) in cells {
        let allowed = y == CONTROL_Y || y == STATUS_Y || y == ROW1_Y || y == ROW3_Y;
        assert!(allowed, "unexpected redraw at ({x},{y})");
        if y == ROW1_Y || y == ROW3_Y {
            assert!((5..14).contains(&x), "only column A redraws, got x={x}");
        }
    }
}

#[test]
fn moving_the_active_cell_changes_only_the_old_and_new_cells_and_the_control_line() {
    let mut s = ready_session();
    let before = frame(&s);
    s.dispatch(Command::NavMoveRelative { dcol: 1, drow: 0 });
    let after = frame(&s);
    for (x, y) in changed(&before, &after) {
        let control = y == CONTROL_Y;
        let header = y == HEADER_Y && (5..23).contains(&x);
        let row1_cells = y == ROW1_Y && (5..23).contains(&x);
        assert!(control || header || row1_cells, "unexpected redraw at ({x},{y})");
    }
}

#[test]
fn a_keystroke_that_changes_nothing_repaints_nothing() {
    let mut s = ready_session();
    let before = frame(&s);
    s.dispatch(Command::NavMoveRelative { dcol: -1, drow: -1 });
    let after = frame(&s);
    assert!(changed(&before, &after).is_empty());
}
