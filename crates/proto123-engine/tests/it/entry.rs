use crate::common::{addr, enter, session, type_keys};
use proto123_engine::{CaretKey, Command, Mode, Then, classify_entry};
use proto123_model::{Alignment, CellContent};
use proto123_value::Value;

fn value_at(s: &proto123_engine::Session, a: &str) -> Value {
    s.calc().value(addr(a))
}

#[test]
fn numbers_typed_in_ready_enter_and_commit_downward() {
    let mut s = session();
    type_keys(&mut s, "120");
    assert_eq!(s.mode(), Mode::Value);
    enter(&mut s);
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(s.active(), addr("A2"));
    type_keys(&mut s, "80");
    enter(&mut s);
    type_keys(&mut s, "Total");
    assert_eq!(s.mode(), Mode::Label);
    s.dispatch(Command::CellCommitEdit(Then::Move { dcol: 1, drow: 0 }));
    s.dispatch(Command::NavSelectCell(addr("A3")));
    type_keys(&mut s, "=SUM(A1:A2)");
    enter(&mut s);
    assert_eq!(value_at(&s, "A3"), Value::Number(200.0));
}

#[test]
fn change_input_recalculates_the_formula() {
    let mut s = session();
    type_keys(&mut s, "120");
    enter(&mut s);
    type_keys(&mut s, "80");
    enter(&mut s);
    s.dispatch(Command::NavSelectCell(addr("A3")));
    type_keys(&mut s, "=SUM(A1:A2)");
    enter(&mut s);
    s.dispatch(Command::NavSelectCell(addr("A1")));
    type_keys(&mut s, "150");
    enter(&mut s);
    assert_eq!(value_at(&s, "A3"), Value::Number(230.0));
}

#[test]
fn quote_prefixes_choose_alignment_and_keep_the_prefix_for_the_formula_bar() {
    let mut s = session();
    type_keys(&mut s, "^Title");
    enter(&mut s);
    let content = s.calc().content(addr("A1")).cloned();
    assert_eq!(
        content,
        Some(CellContent::Label { text: "Title".into(), align: Alignment::Center, prefix: Some('^') })
    );
    assert_eq!(content.map(|c| c.formula_bar_text()), Some("^Title".to_owned()));
}

#[test]
fn f2_edits_with_caret_at_the_end_and_esc_restores() {
    let mut s = session();
    type_keys(&mut s, "2.5");
    enter(&mut s);
    s.dispatch(Command::NavSelectCell(addr("A1")));
    s.dispatch(Command::CellBeginEdit);
    assert_eq!(s.mode(), Mode::Edit);
    let edit = s.edit().cloned().expect("editing");
    assert_eq!(edit.buffer, "2.5");
    assert_eq!(edit.caret, 3);
    s.dispatch(Command::CellEditCaret(CaretKey::Backspace));
    s.dispatch(Command::CellCancelEdit);
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(value_at(&s, "A1"), Value::Number(2.5));
}

#[test]
fn left_in_edit_moves_the_caret_only_and_right_in_value_commits_and_moves() {
    let mut s = session();
    type_keys(&mut s, "12");
    s.dispatch(Command::CellEditCaret(CaretKey::Left));
    assert_eq!(s.mode(), Mode::Value, "still typing");
    assert_eq!(s.edit().map(|e| e.caret), Some(1));
    s.dispatch(Command::CellCommitEdit(Then::Move { dcol: 1, drow: 0 }));
    assert_eq!(s.active(), addr("B1"));
    assert_eq!(value_at(&s, "A1"), Value::Number(12.0));
}

#[test]
fn erasing_the_buffer_clears_the_cell() {
    let mut s = session();
    type_keys(&mut s, "7");
    enter(&mut s);
    s.dispatch(Command::NavSelectCell(addr("A1")));
    s.dispatch(Command::CellBeginEdit);
    s.dispatch(Command::CellEditCaret(CaretKey::End));
    for _ in 0..3 {
        s.dispatch(Command::CellEditCaret(CaretKey::Backspace));
    }
    enter(&mut s);
    assert!(s.calc().content(addr("A1")).is_none());
}

#[test]
fn plain_number_classification() {
    for (text, expect) in [("5", 5.0), ("5.", 5.0), (".5", 0.5), ("-12.75", -12.75), (" 1.5E3 ", 1500.0)] {
        assert_eq!(classify_entry(text), Ok(Some(CellContent::Number(expect))), "{text:?}");
    }
    assert_eq!(classify_entry(""), Ok(None));
}
