use proto123_coord::{CellAddr, RefAddr};
use proto123_engine::{CaretKey, Command, Session, Then};
use proto123_theme::{GridMetrics, high_contrast};
use proto123_tui::{InputMapper, Key, Mods, RawEvent, RawKind, render};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn key(k: Key) -> RawKind {
    RawKind::Key {
        key: k,
        mods: Mods::default(),
    }
}

fn event(kind: RawKind) -> RawEvent {
    RawEvent { t_ms: 0, kind }
}

fn ready_session() -> (Session, proto123_tui::LayoutMap) {
    let mut s = Session::new(high_contrast());
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
    let resize = mapper.map(
        event(RawKind::Resize { cols: 80, rows: 24 }),
        &proto123_tui::LayoutMap::default(),
        &s,
    );
    for c in resize {
        s.dispatch(c);
    }
    let map = render(&s, area, &mut buf);
    (s, map)
}

#[test]
fn arrows_map_to_relative_moves_in_ready() {
    let (s, map) = ready_session();
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
    let cmds = mapper.map(event(key(Key::Right)), &map, &s);
    assert_eq!(cmds, vec![Command::NavMoveRelative { dcol: 1, drow: 0 }]);
}

#[test]
fn printable_key_starts_an_entry_and_enter_commits_downward() {
    let (mut s, map) = ready_session();
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
    for c in mapper.map(event(key(Key::Char('5'))), &map, &s) {
        s.dispatch(c);
    }
    assert_eq!(s.mode(), proto123_engine::Mode::Value);
    let cmds = mapper.map(event(key(Key::Enter)), &map, &s);
    assert_eq!(
        cmds,
        vec![Command::CellCommitEdit(Then::Move { dcol: 0, drow: 1 })]
    );
}

#[test]
fn left_in_edit_is_a_caret_key_not_a_commit() {
    let (mut s, map) = ready_session();
    s.dispatch(Command::CellBeginEdit);
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
    let cmds = mapper.map(event(key(Key::Left)), &map, &s);
    assert_eq!(cmds, vec![Command::CellEditCaret(CaretKey::Left)]);
}

#[test]
fn nothing_mutates_below_the_minimum_size() {
    let (s, _) = ready_session();
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
    let empty = proto123_tui::LayoutMap::default();
    assert!(
        mapper
            .map(event(key(Key::Char('1'))), &empty, &s)
            .is_empty()
    );
    assert!(mapper.map(event(key(Key::Right)), &empty, &s).is_empty());
}

#[test]
fn resize_passes_through_as_visible_grid_size() {
    let s = Session::new(high_contrast());
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
    let cmds = mapper.map(
        event(RawKind::Resize { cols: 80, rows: 24 }),
        &proto123_tui::LayoutMap::default(),
        &s,
    );
    assert_eq!(cmds, vec![Command::SessionResize { cols: 8, rows: 20 }]);
    let below = mapper.map(
        event(RawKind::Resize { cols: 79, rows: 24 }),
        &proto123_tui::LayoutMap::default(),
        &s,
    );
    assert!(below.is_empty());
}

#[test]
fn a1_lookup_helper_is_consistent() {
    assert_eq!(RefAddr::parse("A1").unwrap().addr, CellAddr::ORIGIN);
}
