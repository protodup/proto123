use proto123_coord::{CellAddr, CellRange, RefAddr};
use proto123_engine::{Command, Mode, Session, StringKey, text};
use proto123_theme::high_contrast;
use proto123_tui::{
    Button, HitTarget, InputMapper, Key, LayoutMap, Mods, Pos, RawEvent, RawKind, render,
};
use proto123_value::Value;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn addr(s: &str) -> CellAddr {
    RefAddr::parse(s).unwrap().addr
}

fn range(s: &str) -> CellRange {
    CellRange::parse(s).unwrap()
}

fn setup() -> (Session, InputMapper, LayoutMap) {
    let mut s = Session::new(high_contrast());
    let mut mapper = InputMapper::new(s.theme().metrics);
    let area = Rect::new(0, 0, 80, 24);
    let mut layout = LayoutMap::default();
    for cmd in mapper.map(
        RawEvent {
            t_ms: 0,
            kind: RawKind::Resize { cols: 80, rows: 24 },
        },
        &layout,
        &s,
    ) {
        s.dispatch(cmd);
    }
    let mut buf = Buffer::empty(area);
    layout = render(&s, area, &mut buf);
    (s, mapper, layout)
}

fn feed(
    s: &mut Session,
    mapper: &mut InputMapper,
    layout: &mut LayoutMap,
    t_ms: u64,
    kind: RawKind,
) {
    for cmd in mapper.map(RawEvent { t_ms, kind }, layout, s) {
        s.dispatch(cmd);
    }
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    *layout = render(s, area, &mut buf);
}

fn cell_pos(layout: &LayoutMap, target_addr: CellAddr) -> Pos {
    for (r, t) in layout.entries() {
        if *t == HitTarget::GridCell(target_addr) {
            return Pos::Cell { col: r.x, row: r.y };
        }
    }
    panic!("cell {target_addr} not drawn");
}

fn menu_pos(layout: &LayoutMap, label: &str, s: &Session) -> Pos {
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    render(s, area, &mut buf);
    for (r, t) in layout.entries() {
        if matches!(t, HitTarget::MenuItem { .. }) {
            let drawn: String = (r.x..r.x + r.width)
                .filter_map(|x| buf.cell((x, r.y)).map(|c| c.symbol().to_owned()))
                .collect();
            if drawn.contains(label) {
                return Pos::Cell { col: r.x, row: r.y };
            }
        }
    }
    panic!("menu item {label:?} not found");
}

#[test]
fn click_shift_click_and_double_click_timing_follow_500ms_rule() {
    let (mut s, mut m, mut l) = setup();
    let c5 = cell_pos(&l, addr("C5"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        100,
        RawKind::PointerDown {
            at: c5,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        120,
        RawKind::PointerUp {
            at: c5,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.active(), addr("C5"));

    let e9 = cell_pos(&l, addr("E9"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        200,
        RawKind::PointerDown {
            at: e9,
            button: Button::Left,
            mods: Mods {
                shift: true,
                ..Mods::default()
            },
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        220,
        RawKind::PointerUp {
            at: e9,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.selection(), range("C5:E9"));

    // 501 ms later on E9 -> single click, NOT double click.
    feed(
        &mut s,
        &mut m,
        &mut l,
        701,
        RawKind::PointerDown {
            at: e9,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        720,
        RawKind::PointerUp {
            at: e9,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Ready);

    // 200 ms later on E9 -> double click enters EDIT with caret at end.
    feed(
        &mut s,
        &mut m,
        &mut l,
        901,
        RawKind::PointerDown {
            at: e9,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        920,
        RawKind::PointerUp {
            at: e9,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Edit);
}

#[test]
fn drag_selects_range_and_auto_scrolls_past_bottom_edge() {
    let (mut s, mut m, mut l) = setup();
    let b2 = cell_pos(&l, addr("B2"));
    let d6 = cell_pos(&l, addr("D6"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        10,
        RawKind::PointerDown {
            at: b2,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        20,
        RawKind::PointerMove {
            at: d6,
            buttons_down: true,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.selection(), range("B2:D6"));

    // Drag past bottom edge (row 23 is status bar, outside grid_rect).
    feed(
        &mut s,
        &mut m,
        &mut l,
        30,
        RawKind::PointerMove {
            at: Pos::Cell { col: 14, row: 23 },
            buttons_down: true,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        40,
        RawKind::PointerUp {
            at: Pos::Cell { col: 14, row: 23 },
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert!(
        s.viewport().top >= 1,
        "viewport scrolled down: {:?}",
        s.viewport()
    );
    assert!(
        s.selection().end().row() >= 20,
        "selection extended past row 20: {:?}",
        s.selection()
    );
}

#[test]
fn wheel_scrolls_three_rows_and_shift_wheel_scrolls_columns() {
    let (mut s, mut m, mut l) = setup();
    feed(
        &mut s,
        &mut m,
        &mut l,
        10,
        RawKind::Wheel {
            dx: 0,
            dy: 1,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.viewport().top, 3);
    assert_eq!(s.viewport().left, 0);

    feed(
        &mut s,
        &mut m,
        &mut l,
        20,
        RawKind::Wheel {
            dx: 0,
            dy: 1,
            mods: Mods {
                shift: true,
                ..Mods::default()
            },
        },
    );
    assert_eq!(s.viewport().left, 3);

    feed(
        &mut s,
        &mut m,
        &mut l,
        30,
        RawKind::Wheel {
            dx: 1,
            dy: 0,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.viewport().left, 6);
}

#[test]
fn formula_bar_click_positions_caret_and_pointing_updates_live_then_click_commits() {
    let (mut s, mut m, mut l) = setup();
    for c in "12345".chars() {
        feed(
            &mut s,
            &mut m,
            &mut l,
            10,
            RawKind::Key {
                key: Key::Char(c),
                mods: Mods::default(),
            },
        );
    }
    feed(
        &mut s,
        &mut m,
        &mut l,
        11,
        RawKind::Key {
            key: Key::Enter,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        12,
        RawKind::Key {
            key: Key::Up,
            mods: Mods::default(),
        },
    );

    // Formula bar starts at col 9 ("12345"). Click at char index 2 (col 11) -> EDIT with caret 2.
    feed(
        &mut s,
        &mut m,
        &mut l,
        20,
        RawKind::PointerDown {
            at: Pos::Cell { col: 11, row: 0 },
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Edit);
    assert_eq!(s.edit().map(|e| e.caret), Some(2));
    feed(
        &mut s,
        &mut m,
        &mut l,
        21,
        RawKind::Key {
            key: Key::Esc,
            mods: Mods::default(),
        },
    );

    // Move to B1, type `=SUM(`, click B2 and drag to B9 -> POINT with `=SUM(B2:B9`.
    let b1 = cell_pos(&l, addr("B1"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        30,
        RawKind::PointerDown {
            at: b1,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        31,
        RawKind::PointerUp {
            at: b1,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    for c in "=SUM(".chars() {
        feed(
            &mut s,
            &mut m,
            &mut l,
            32,
            RawKind::Key {
                key: Key::Char(c),
                mods: Mods::default(),
            },
        );
    }
    let b2 = cell_pos(&l, addr("B2"));
    let b9 = cell_pos(&l, addr("B9"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        40,
        RawKind::PointerDown {
            at: b2,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        41,
        RawKind::PointerMove {
            at: b9,
            buttons_down: true,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        42,
        RawKind::PointerUp {
            at: b9,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Point);
    assert_eq!(s.edit().map(|e| e.buffer.as_str()), Some("=SUM(B2:B9"));

    // Type `)` -> returns to VALUE keeping the reference; click C1 -> commits and selects C1.
    feed(
        &mut s,
        &mut m,
        &mut l,
        50,
        RawKind::Key {
            key: Key::Char(')'),
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Value);
    let c1 = cell_pos(&l, addr("C1"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        60,
        RawKind::PointerDown {
            at: c1,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        61,
        RawKind::PointerUp {
            at: c1,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(s.active(), addr("C1"));
    assert_eq!(
        s.calc().content(addr("B1")).map(|c| c.formula_bar_text()),
        Some("=SUM(B2:B9)".to_owned())
    );
}

#[test]
fn menu_hover_click_and_click_outside_and_prompt_grid_drag_work() {
    let (mut s, mut m, mut l) = setup();
    for c in "77".chars() {
        feed(
            &mut s,
            &mut m,
            &mut l,
            10,
            RawKind::Key {
                key: Key::Char(c),
                mods: Mods::default(),
            },
        );
    }
    feed(
        &mut s,
        &mut m,
        &mut l,
        11,
        RawKind::Key {
            key: Key::Enter,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.calc().value(addr("A1")), Value::Number(77.0));

    // Hover over Range in root bar -> highlights and shows row 2 preview.
    let range_pos = menu_pos(&l, "Range", &s);
    feed(
        &mut s,
        &mut m,
        &mut l,
        20,
        RawKind::PointerMove {
            at: range_pos,
            buttons_down: false,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.root_hover(), Some(2));
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    let row2: String = (0..80)
        .filter_map(|x| buf.cell((x, 2)).map(|c| c.symbol().to_owned()))
        .collect();
    assert!(row2.contains("Erase") && row2.contains("Copy"), "{row2}");

    // Click Edit in root bar -> opens Edit submenu; click outside -> closes menu cleanly.
    let edit_pos = menu_pos(&l, "Edit", &s);
    feed(
        &mut s,
        &mut m,
        &mut l,
        30,
        RawKind::PointerDown {
            at: edit_pos,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        31,
        RawKind::PointerUp {
            at: edit_pos,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Menu);
    let outside = cell_pos(&l, addr("D4"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        32,
        RawKind::PointerDown {
            at: outside,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        33,
        RawKind::PointerUp {
            at: outside,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Ready);

    // Click Edit then click Undo -> undoes A1=77.
    let edit_pos = menu_pos(&l, "Edit", &s);
    feed(
        &mut s,
        &mut m,
        &mut l,
        40,
        RawKind::PointerDown {
            at: edit_pos,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        41,
        RawKind::PointerUp {
            at: edit_pos,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    let undo_pos = menu_pos(&l, text(StringKey::MenuEditUndo), &s);
    feed(
        &mut s,
        &mut m,
        &mut l,
        42,
        RawKind::PointerDown {
            at: undo_pos,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        43,
        RawKind::PointerUp {
            at: undo_pos,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(s.calc().value(addr("A1")), Value::Empty);

    // Prompt grid click/drag fills range text.
    s.dispatch(Command::RangeCopy);
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    l = render(&s, area, &mut buf);
    let b2 = cell_pos(&l, addr("B2"));
    let c3 = cell_pos(&l, addr("C3"));
    feed(
        &mut s,
        &mut m,
        &mut l,
        50,
        RawKind::PointerDown {
            at: b2,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        51,
        RawKind::PointerMove {
            at: c3,
            buttons_down: true,
            mods: Mods::default(),
        },
    );
    feed(
        &mut s,
        &mut m,
        &mut l,
        52,
        RawKind::PointerUp {
            at: c3,
            button: Button::Left,
            mods: Mods::default(),
        },
    );
    assert_eq!(s.prompt().map(|p| p.text()), Some("B2:C3"));
}

#[test]
fn pointer_events_below_minimum_size_produce_no_commands() {
    let (s, mut m, _) = setup();
    let empty = LayoutMap::default();
    let cmds = m.map(
        RawEvent {
            t_ms: 10,
            kind: RawKind::PointerDown {
                at: Pos::Cell { col: 10, row: 5 },
                button: Button::Left,
                mods: Mods::default(),
            },
        },
        &empty,
        &s,
    );
    assert!(cmds.is_empty());
    let wheel = m.map(
        RawEvent {
            t_ms: 11,
            kind: RawKind::Wheel {
                dx: 0,
                dy: 1,
                mods: Mods::default(),
            },
        },
        &empty,
        &s,
    );
    assert!(wheel.is_empty());
}
