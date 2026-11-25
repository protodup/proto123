use crate::common::{addr, enter, session, type_keys};
use proto123_coord::CellRange;
use proto123_engine::{CaretKey, Command, Mode, PromptButton, StringKey, Then};
use proto123_theme::high_contrast;
use proto123_tui::{InputMapper, Key, Mods, RawEvent, RawKind, render};
use proto123_value::Value;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn range(s: &str) -> CellRange {
    CellRange::parse(s).unwrap()
}

fn line(buf: &Buffer, y: u16) -> String {
    (0..buf.area().width)
        .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_owned()))
        .collect()
}

#[test]
fn shift_arrow_and_f8_extend_selection_and_formula_bar_shows_range() {
    let mut s = session();
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });

    // Shift+Down twice selects A1:A3 and shows A1:A3 in formula bar.
    s.dispatch(Command::NavExtendSelection(addr("A2")));
    s.dispatch(Command::NavExtendSelection(addr("A3")));
    assert_eq!(s.selection(), range("A1:A3"));
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    assert!(line(&buf, 0).contains("A1:A3"), "{}", line(&buf, 0));

    // Plain arrow resets selection; F8 sets [EXT] and plain arrows extend via InputMapper.
    s.dispatch(Command::NavMoveRelative { dcol: 0, drow: -1 });
    assert_eq!(s.selection(), range("A2:A2"));

    let mut mapper = InputMapper::new(high_contrast().metrics);
    let layout = render(&s, area, &mut buf);
    for cmd in mapper.map(
        RawEvent {
            t_ms: 1,
            kind: RawKind::Key {
                key: Key::F(8),
                mods: Mods::default(),
            },
        },
        &layout,
        &s,
    ) {
        s.dispatch(cmd);
    }
    assert!(s.extend_mode());
    let layout = render(&s, area, &mut buf);
    assert!(line(&buf, 23).contains("[EXT]"), "{}", line(&buf, 23));

    for cmd in mapper.map(
        RawEvent {
            t_ms: 2,
            kind: RawKind::Key {
                key: Key::Down,
                mods: Mods::default(),
            },
        },
        &layout,
        &s,
    ) {
        s.dispatch(cmd);
    }
    assert_eq!(s.selection(), range("A2:A3"));

    // F8 again or Esc ends extend mode while keeping the selection.
    for cmd in mapper.map(
        RawEvent {
            t_ms: 3,
            kind: RawKind::Key {
                key: Key::Esc,
                mods: Mods::default(),
            },
        },
        &layout,
        &s,
    ) {
        s.dispatch(cmd);
    }
    assert!(!s.extend_mode());
    assert_eq!(s.selection(), range("A2:A3"));
}

#[test]
fn delete_erases_selection_and_undo_redo_restore_without_nav_steps() {
    let mut s = session();
    // Undo on empty history sets status.nothingToUndo.
    let out = s.dispatch(Command::EditUndo);
    assert_eq!(
        s.status().map(|st| st.key),
        Some(StringKey::StatusNothingToUndo)
    );
    assert_eq!(out.redraw, proto123_engine::Redraw::Regions);

    type_keys(&mut s, "10");
    enter(&mut s);
    type_keys(&mut s, "20");
    enter(&mut s);
    s.dispatch(Command::NavSelectCell(addr("A1")));
    s.dispatch(Command::NavExtendSelection(addr("A2")));

    // Navigation and selection create no extra undo steps beyond the 2 edits.
    s.dispatch(Command::RangeErase);
    assert_eq!(s.calc().value(addr("A1")), Value::Empty);
    assert_eq!(s.calc().value(addr("A2")), Value::Empty);

    // Undo restores both erased cells in 1 step.
    s.dispatch(Command::EditUndo);
    assert_eq!(s.calc().value(addr("A1")), Value::Number(10.0));
    assert_eq!(s.calc().value(addr("A2")), Value::Number(20.0));

    // Redo erases them again.
    s.dispatch(Command::EditRedo);
    assert_eq!(s.calc().value(addr("A1")), Value::Empty);
    assert_eq!(s.calc().value(addr("A2")), Value::Empty);

    // Undo back to A1=10, A2=20, then undo A2, then undo A1 -> nothing left to undo.
    s.dispatch(Command::EditUndo);
    s.dispatch(Command::EditUndo);
    s.dispatch(Command::EditUndo);
    assert!(s.calc().workbook().is_empty());
    s.dispatch(Command::EditUndo);
    assert_eq!(
        s.status().map(|st| st.key),
        Some(StringKey::StatusNothingToUndo)
    );
}

#[test]
fn copy_and_move_prompts_prefill_accept_typed_and_pointed_refs_and_esc_cancels() {
    let mut s = session();
    type_keys(&mut s, "5");
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::NavSelectCell(addr("B1")));
    type_keys(&mut s, "=A1*2");
    s.dispatch(Command::CellCommitEdit(Then::Stay));

    // Open Copy prompt pre-filled with B1, submit, type B2:B5, submit.
    s.dispatch(Command::RangeCopy);
    assert_eq!(s.mode(), Mode::Dialog);
    assert_eq!(s.prompt().map(|p| p.text()), Some("B1"));
    s.dispatch(Command::PromptSubmit);
    for c in "B2:B5".chars() {
        s.dispatch(Command::PromptInput(c));
    }
    s.dispatch(Command::PromptSubmit);
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(
        s.calc().content(addr("B5")).map(|c| c.formula_bar_text()),
        Some("=A5*2".to_owned())
    );
    assert_eq!(s.calc().value(addr("B1")), Value::Number(10.0));

    // Pointing with arrows in Move prompt, then Esc leaves no change.
    s.dispatch(Command::NavSelectCell(addr("B1")));
    s.dispatch(Command::NavExtendSelection(addr("B2")));
    s.dispatch(Command::RangeMove);
    assert_eq!(s.prompt().map(|p| p.text()), Some("B1:B2"));
    s.dispatch(Command::PromptSubmit);
    s.dispatch(Command::PointSetReference {
        anchor: None,
        cursor: addr("D1"),
    });
    assert_eq!(s.prompt().map(|p| p.text()), Some("D1"));
    s.dispatch(Command::PromptCancel);
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(s.calc().value(addr("D1")), Value::Empty);
    assert_eq!(s.calc().value(addr("B1")), Value::Number(10.0));

    // Cancel button focus also cancels on submit.
    s.dispatch(Command::RangeCopy);
    s.dispatch(Command::PromptFocus(PromptButton::Cancel));
    s.dispatch(Command::PromptSubmit);
    assert_eq!(s.mode(), Mode::Ready);

    // Invalid range text gives status diagnostic without closing dialog.
    s.dispatch(Command::RangeCopy);
    s.dispatch(Command::PromptInput('?'));
    s.dispatch(Command::PromptSubmit);
    assert_eq!(s.mode(), Mode::Dialog);
    assert_eq!(
        s.status().map(|st| st.key),
        Some(StringKey::DiagInvalidRange)
    );
    s.dispatch(Command::PromptCaret(CaretKey::Backspace));
    s.dispatch(Command::PromptCancel);
    assert_eq!(s.mode(), Mode::Ready);
}
