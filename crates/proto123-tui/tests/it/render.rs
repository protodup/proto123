use proto123_coord::{CellAddr, RefAddr};
use proto123_engine::{Command, Session, Then};
use proto123_theme::high_contrast;
use proto123_tui::{HitTarget, LayoutMap, render};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn frame(session: &Session, cols: u16, rows: u16) -> (Buffer, LayoutMap) {
    let area = Rect::new(0, 0, cols, rows);
    let mut buf = Buffer::empty(area);
    let map = render(session, area, &mut buf);
    (buf, map)
}

fn line(buf: &Buffer, y: u16) -> String {
    let area = *buf.area();
    (0..area.width)
        .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_owned()))
        .collect()
}

fn addr(s: &str) -> CellAddr {
    RefAddr::parse(s).unwrap().addr
}

fn sized_session() -> Session {
    let mut s = Session::new(high_contrast());
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s
}

#[test]
fn fresh_session_shows_address_badge_and_bracketed_headers() {
    let s = sized_session();
    let (buf, _) = frame(&s, 80, 24);
    assert!(
        line(&buf, 0).starts_with("[A1]"),
        "control line: {}",
        line(&buf, 0)
    );
    assert!(line(&buf, 0).trim_end().ends_with("READY"));
    assert!(
        line(&buf, 2).contains("[A]"),
        "column header: {}",
        line(&buf, 2)
    );
    assert!(
        line(&buf, 3).starts_with("[  1]"),
        "row header: {}",
        line(&buf, 3)
    );
}

#[test]
fn layout_map_resolves_the_drawn_cell_after_scrolling() {
    let mut s = sized_session();
    s.dispatch(Command::NavSelectCell(addr("B3")));
    let (buf, map) = frame(&s, 80, 24);
    assert!(line(&buf, 0).starts_with("[B3]"));
    // B3 is at header column x = 5 + 9, row y = 3 + 2.
    assert_eq!(map.hit(14, 5), HitTarget::GridCell(addr("B3")));
    assert_eq!(map.hit(0, 0), HitTarget::AddressBox);
}

#[test]
fn below_minimum_shows_the_enlarge_message_and_an_empty_map() {
    let s = sized_session();
    let (buf, map) = frame(&s, 79, 24);
    assert!(line(&buf, 0).contains("Enlarge the window"));
    assert!(map.is_empty());
}

#[test]
fn entry_buffer_and_formula_bar_show_the_label_prefix() {
    let mut s = sized_session();
    s.dispatch(Command::CellBeginEntry('^'));
    for c in "Title".chars() {
        s.dispatch(Command::CellEditInput(c));
    }
    let (buf, _) = frame(&s, 80, 24);
    assert!(line(&buf, 0).contains("^Title"), "{}", line(&buf, 0));
    assert!(line(&buf, 0).trim_end().ends_with("LABEL"));
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    let (buf, _) = frame(&s, 80, 24);
    // Row 1 is the first grid row (y = 3); A1 holds the label.
    assert!(!line(&buf, 3).contains("^Title"), "grid omits the prefix");
    assert!(line(&buf, 3).contains("Title"), "{}", line(&buf, 3));
}

#[test]
fn numbers_are_right_aligned_in_nine_columns() {
    let mut s = sized_session();
    s.dispatch(Command::CellBeginEntry('1'));
    s.dispatch(Command::CellEditInput('2'));
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    let (buf, _) = frame(&s, 80, 24);
    // Column A is 9 wide after the 5-wide row header, so the number ends at column 13.
    assert!(
        line(&buf, 3).starts_with("[  1]       12"),
        "{:?}",
        line(&buf, 3)
    );
}
