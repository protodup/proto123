use proto123_engine::{Command, Session, Then};
use proto123_theme::high_contrast;
use proto123_tui::render;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn row_text(s: &Session, y: u16) -> String {
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    render(s, area, &mut buf);
    (0..80)
        .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_owned()))
        .collect()
}

fn session_with(label: &str, at_col: u16) -> Session {
    let mut s = Session::new(high_contrast());
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    for _ in 0..at_col {
        s.dispatch(Command::NavMoveRelative { dcol: 1, drow: 0 });
    }
    s.dispatch(Command::CellBeginEntry('\''));
    for c in label.chars() {
        s.dispatch(Command::CellEditInput(c));
    }
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s
}

#[test]
fn long_label_spills_over_empty_neighbours() {
    let s = session_with("Quarterly total", 0);
    assert!(
        row_text(&s, 3).contains("Quarterly total"),
        "{}",
        row_text(&s, 3)
    );
}

#[test]
fn long_label_is_cut_at_the_first_non_empty_neighbour() {
    let mut s = session_with("Quarterly total", 0);
    s.dispatch(Command::NavMoveRelative { dcol: 1, drow: 0 });
    s.dispatch(Command::CellBeginEntry('X'));
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    let text = row_text(&s, 3);
    assert!(text.contains("Quarter"), "{text}");
    assert!(!text.contains("Quarterly total"), "{text}");
}

#[test]
fn oversized_number_never_shows_truncated_digits() {
    let mut s = Session::new(high_contrast());
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s.dispatch(Command::CellBeginEntry('1'));
    for c in "23456789012".chars() {
        s.dispatch(Command::CellEditInput(c));
    }
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    let text = row_text(&s, 3);
    assert!(text.contains("1.235E+11"), "{text}");
}

#[test]
fn error_tokens_use_the_error_token_colour_and_the_error_status_too() {
    let mut s = Session::new(high_contrast());
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s.dispatch(Command::CellBeginEntry('='));
    for c in "10/B1".chars() {
        s.dispatch(Command::CellEditInput(c));
    }
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::NavMoveRelative { dcol: 1, drow: 0 });
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    // "#DIV/0!" is right-aligned in A1's 9 columns: starts at x = 5 + 2.
    let cell = buf.cell((7, 3)).expect("A1 drawn");
    assert_eq!(cell.symbol(), "#");
    assert_eq!(
        cell.fg,
        ratatui::style::Color::Rgb(255, 255, 0),
        "error token colour"
    );

    // A syntax error puts the error status in the error token colour as well.
    s.dispatch(Command::CellBeginEntry('='));
    s.dispatch(Command::CellEditInput('('));
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    let status = buf.cell((0, 23)).expect("status drawn");
    assert_eq!(
        status.fg,
        ratatui::style::Color::Rgb(255, 255, 0),
        "error status colour"
    );
}
