use crate::common::{addr, session, type_keys};
use proto123_engine::{Command, Mode, Status, StatusKind, StringKey, Then, text};
use proto123_value::Value;

#[test]
fn unparsable_commit_stays_in_edit_with_caret_at_the_problem() {
    let mut s = session();
    type_keys(&mut s, "123 Main St");
    s.dispatch(Command::CellCommitEdit(Then::Move { dcol: 0, drow: 1 }));
    assert_eq!(s.mode(), Mode::Value);
    assert_eq!(s.active(), addr("A1"));
    let status = s.status().copied().expect("diagnostic shown");
    assert_eq!(status.key, StringKey::DiagSyntax);
    assert_eq!(status.kind, StatusKind::Error);
    assert_eq!(status.hint, Some(StringKey::DiagUseQuoteForText));
    assert!(s.edit().is_some());
}

#[test]
fn quote_prefix_stores_the_text_as_a_left_label() {
    let mut s = session();
    type_keys(&mut s, "'123 Main St");
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    assert_eq!(
        s.calc().value(addr("A1")),
        Value::Text("123 Main St".into())
    );
}

#[test]
fn scientific_entry_stores_a_number() {
    let mut s = session();
    type_keys(&mut s, "1.5E3");
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    assert_eq!(s.calc().value(addr("A1")), Value::Number(1500.0));
}

#[test]
fn escape_abandons_a_bad_entry() {
    let mut s = session();
    type_keys(&mut s, "=SUM(");
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::CellCancelEdit);
    assert_eq!(s.mode(), Mode::Ready);
    assert!(s.calc().content(addr("A1")).is_none());
}

#[test]
fn status_message_fills_the_position() {
    let st = Status {
        key: StringKey::DiagSyntax,
        pos: Some(3),
        hint: None,
        kind: StatusKind::Error,
    };
    assert!(st.message().contains("character 4"), "{}", st.message());
    assert!(text(StringKey::BadgeReady) == "READY");
}
