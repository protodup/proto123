use crate::common::{addr, enter, session, type_keys};
use proto123_engine::{CATALOG, Command, Mode, PromptButton, Redraw, StringKey, text};
use proto123_theme::ThemeId;
use proto123_tui::{InputMapper, Key, Mods, RawEvent, RawKind, render};
use proto123_value::Value;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn send_key(s: &mut proto123_engine::Session, mapper: &mut InputMapper, key: Key) -> Redraw {
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    let layout = render(s, area, &mut buf);
    let mut redraw = Redraw::None;
    for cmd in mapper.map(
        RawEvent {
            t_ms: 1,
            kind: RawKind::Key {
                key,
                mods: Mods::default(),
            },
        },
        &layout,
        s,
    ) {
        let out = s.dispatch(cmd);
        if out.redraw == Redraw::Full {
            redraw = Redraw::Full;
        } else if out.redraw == Redraw::Regions && redraw == Redraw::None {
            redraw = Redraw::Regions;
        }
    }
    redraw
}

#[test]
fn slash_opens_menu_arrows_wrap_letters_activate_and_esc_steps_up() {
    let mut s = session();
    let mut mapper = InputMapper::new(s.theme().metrics);

    // `/` opens MENU at root with first item (File) highlighted and described on line 2.
    send_key(&mut s, &mut mapper, Key::Char('/'));
    assert_eq!(s.mode(), Mode::Menu);
    let menu = s.menu().unwrap();
    assert_eq!(menu.path, Vec::<usize>::new());
    assert_eq!(menu.highlighted, 0);
    assert_eq!(menu.second_line_text(CATALOG), "New");

    // Left wraps around to View (index 3), Right wraps back to File (index 0).
    send_key(&mut s, &mut mapper, Key::Left);
    assert_eq!(s.menu().unwrap().highlighted, 3);
    assert_eq!(s.menu().unwrap().second_line_text(CATALOG), "Theme");
    send_key(&mut s, &mut mapper, Key::Right);
    assert_eq!(s.menu().unwrap().highlighted, 0);

    // Enter opens File submenu; Esc goes up to root; Esc again goes to READY.
    send_key(&mut s, &mut mapper, Key::Enter);
    assert_eq!(s.menu().unwrap().path, vec![0]);
    assert_eq!(
        s.menu().unwrap().second_line_text(CATALOG),
        text(StringKey::MenuFileNewDesc)
    );
    send_key(&mut s, &mut mapper, Key::Esc);
    assert_eq!(s.mode(), Mode::Menu);
    assert!(s.menu().unwrap().path.is_empty());
    send_key(&mut s, &mut mapper, Key::Esc);
    assert_eq!(s.mode(), Mode::Ready);

    // `/ R E` erases the selection.
    type_keys(&mut s, "42");
    s.dispatch(Command::CellCommitEdit(proto123_engine::Then::Stay));
    assert_eq!(s.calc().value(addr("A1")), Value::Number(42.0));
    send_key(&mut s, &mut mapper, Key::Char('/'));
    send_key(&mut s, &mut mapper, Key::Char('R'));
    send_key(&mut s, &mut mapper, Key::Char('E'));
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(s.calc().value(addr("A1")), Value::Empty);

    // `/ R C` opens the Copy prompt.
    send_key(&mut s, &mut mapper, Key::Char('/'));
    send_key(&mut s, &mut mapper, Key::Char('r'));
    send_key(&mut s, &mut mapper, Key::Char('c'));
    assert_eq!(s.mode(), Mode::Dialog);
    send_key(&mut s, &mut mapper, Key::Esc);
    assert_eq!(s.mode(), Mode::Ready);
}

#[test]
fn file_new_prompts_only_when_non_empty_defaults_to_no_and_clears_undo() {
    let mut s = session();
    let mut mapper = InputMapper::new(s.theme().metrics);

    // Empty workbook -> New executes without a prompt.
    send_key(&mut s, &mut mapper, Key::Char('/'));
    send_key(&mut s, &mut mapper, Key::Char('F'));
    send_key(&mut s, &mut mapper, Key::Char('N'));
    assert_eq!(s.mode(), Mode::Ready);

    // Non-empty workbook -> New opens confirmation prompt with default No.
    type_keys(&mut s, "99");
    enter(&mut s);
    assert!(s.can_undo());

    send_key(&mut s, &mut mapper, Key::Char('/'));
    send_key(&mut s, &mut mapper, Key::Char('F'));
    send_key(&mut s, &mut mapper, Key::Char('N'));
    assert_eq!(s.mode(), Mode::Dialog);
    assert_eq!(s.prompt().and_then(|p| p.focus()), Some(PromptButton::No));

    // Pressing Enter immediately chooses default (No / Cancel), keeping content.
    send_key(&mut s, &mut mapper, Key::Enter);
    assert_eq!(s.mode(), Mode::Ready);
    assert_eq!(s.calc().value(addr("A1")), Value::Number(99.0));

    // Confirming with Tab -> Enter clears workbook and undo history.
    send_key(&mut s, &mut mapper, Key::Char('/'));
    send_key(&mut s, &mut mapper, Key::Char('F'));
    send_key(&mut s, &mut mapper, Key::Char('N'));
    send_key(&mut s, &mut mapper, Key::Tab);
    assert_eq!(s.prompt().and_then(|p| p.focus()), Some(PromptButton::Yes));
    send_key(&mut s, &mut mapper, Key::Enter);
    assert_eq!(s.mode(), Mode::Ready);
    assert!(s.calc().workbook().is_empty());
    assert!(!s.can_undo());

    s.dispatch(Command::EditUndo);
    assert_eq!(
        s.status().map(|st| st.key),
        Some(StringKey::StatusNothingToUndo)
    );
}

#[test]
fn view_theme_cycles_all_builtin_themes_with_full_redraw_and_no_undo_step() {
    let mut s = session();
    let mut mapper = InputMapper::new(s.theme().metrics);
    assert_eq!(s.theme().id, ThemeId::HighContrast);

    let mut seen = vec![s.theme().id];
    for _ in 0..3 {
        send_key(&mut s, &mut mapper, Key::Char('/'));
        send_key(&mut s, &mut mapper, Key::Char('V'));
        let redraw = send_key(&mut s, &mut mapper, Key::Char('T'));
        assert_eq!(redraw, Redraw::Full);
        assert_eq!(s.mode(), Mode::Ready);
        assert!(!s.can_undo());
        seen.push(s.theme().id);
    }
    assert_eq!(
        seen,
        vec![
            ThemeId::HighContrast,
            ThemeId::Console16,
            ThemeId::Mono,
            ThemeId::HighContrast
        ]
    );
}
