use crate::common::{addr, enter, session, type_keys};
use proto123_engine::{Command, Then};

#[test]
fn moving_to_formula_cell_announces_exact_contract_and_full_precision() {
    let mut s = session();
    for (cell, val) in [("C1", "50"), ("C2", "50"), ("C3", "50")] {
        s.dispatch(Command::NavSelectCell(addr(cell)));
        type_keys(&mut s, val);
        enter(&mut s);
    }
    s.dispatch(Command::NavSelectCell(addr("C4")));
    type_keys(&mut s, "=SUM(C1:C3)");
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::NavSelectCell(addr("A1")));

    // Moving to C4 gives the exact contract announcement.
    let out = s.dispatch(Command::NavSelectCell(addr("C4")));
    assert_eq!(
        out.announce.as_deref(),
        Some("C4, formula =SUM(C1:C3), value 150, READY")
    );

    // Numbers use format_full (=0.1+0.2 -> value 0.3).
    s.dispatch(Command::NavSelectCell(addr("D1")));
    type_keys(&mut s, "=0.1+0.2");
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::NavSelectCell(addr("A1")));
    let out = s.dispatch(Command::NavSelectCell(addr("D1")));
    assert_eq!(
        out.announce.as_deref(),
        Some("D1, formula =0.1+0.2, value 0.3, READY")
    );
}

#[test]
fn mode_menu_highlight_status_and_unchanged_announcements() {
    let mut s = session();

    // No-op move at A1 -> no announcement.
    let noop = s.dispatch(Command::NavMoveRelative { dcol: -1, drow: 0 });
    assert_eq!(noop.announce, None);

    // Mode change emits announcement.
    let begin = s.dispatch(Command::CellBeginEntry('1'));
    assert!(
        begin
            .announce
            .as_deref()
            .is_some_and(|a| a.ends_with(", VALUE"))
    );
    s.dispatch(Command::CellCancelEdit);

    // Menu highlight in Range -> Copy emits exact menu announcement.
    s.dispatch(Command::MenuOpen);
    s.dispatch(Command::MenuOpenAt(2)); // Range
    let copy_out = s.dispatch(Command::MenuMove(1)); // Copy
    assert_eq!(
        copy_out.announce.as_deref(),
        Some("Menu Range, Copy: copy a range to another place, MENU")
    );
    s.dispatch(Command::MenuClose);

    // Status/error triggers an announcement even when cell and mode stay the same.
    let err_out = s.dispatch(Command::EditUndo);
    assert_eq!(
        err_out.announce.as_deref(),
        Some("A1, empty, READY, Nothing to undo")
    );
}
