use proto123_coord::{CellAddr, RefAddr};
use proto123_engine::{Command, Session, Then};
use proto123_theme::{CapabilityProfile, GlyphSet, Theme, ThemeId, for_id};
use proto123_tui::render;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

fn addr(s: &str) -> CellAddr {
    RefAddr::parse(s).unwrap().addr
}

fn is_named_ansi16(c: Color) -> bool {
    matches!(
        c,
        Color::Reset
            | Color::Black
            | Color::Red
            | Color::Green
            | Color::Yellow
            | Color::Blue
            | Color::Magenta
            | Color::Cyan
            | Color::Gray
            | Color::DarkGray
            | Color::LightRed
            | Color::LightGreen
            | Color::LightYellow
            | Color::LightBlue
            | Color::LightMagenta
            | Color::LightCyan
            | Color::White
    )
}

fn is_allowed_box_glyph(ch: char) -> bool {
    ch.is_ascii()
        || matches!(
            ch,
            '─' | '│' | '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼'
        )
}

fn frame_text(buf: &Buffer) -> String {
    let area = *buf.area();
    let mut out = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buf.cell((x, y)) {
                out.push_str(cell.symbol());
            }
        }
        out.push('\n');
    }
    out
}

fn populate_states(theme: Theme) -> Vec<(String, Buffer)> {
    let area = Rect::new(0, 0, 80, 24);
    let mut frames = Vec::new();

    // 1. Grid with error token (#DIV/0!) and multi-cell selection + [EXT].
    let mut s = Session::new(theme);
    s.dispatch(Command::SessionResize { cols: 8, rows: 20 });
    s.dispatch(Command::CellBeginEntry('='));
    for c in "10/B1".chars() {
        s.dispatch(Command::CellEditInput(c));
    }
    s.dispatch(Command::CellCommitEdit(Then::Stay));
    s.dispatch(Command::NavToggleExtendMode);
    s.dispatch(Command::NavExtendSelection(addr("A3")));
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    frames.push(("selection_ext_error".to_owned(), buf));

    // 2. Open Slash Command Bar menu.
    s.dispatch(Command::NavSelectCell(addr("A1")));
    s.dispatch(Command::MenuOpen);
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    frames.push(("menu_open".to_owned(), buf));

    // 3. Prompt dialog.
    s.dispatch(Command::MenuTrigger('R'));
    s.dispatch(Command::MenuTrigger('C'));
    let mut buf = Buffer::empty(area);
    render(&s, area, &mut buf);
    frames.push(("dialog".to_owned(), buf));

    frames
}

#[test]
fn console_profile_frames_use_only_ansi16_no_bold_and_allowed_glyphs() {
    let console_theme = CapabilityProfile::detect(Some("linux"), None).theme();
    for (name, buf) in populate_states(console_theme) {
        let area = *buf.area();
        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buf.cell((x, y)).unwrap();
                assert!(
                    is_named_ansi16(cell.fg),
                    "{name} ({x},{y}) fg {:?}",
                    cell.fg
                );
                assert!(
                    is_named_ansi16(cell.bg),
                    "{name} ({x},{y}) bg {:?}",
                    cell.bg
                );
                assert!(
                    !cell.modifier.contains(Modifier::BOLD)
                        && !cell.modifier.contains(Modifier::UNDERLINED),
                    "{name} ({x},{y}) modifier {:?}",
                    cell.modifier
                );
                for ch in cell.symbol().chars() {
                    assert!(
                        is_allowed_box_glyph(ch),
                        "{name} ({x},{y}) disallowed glyph {ch:?}"
                    );
                }
            }
        }
    }

    let ascii_theme = CapabilityProfile::detect(Some("dumb"), None).theme();
    assert_eq!(ascii_theme.glyphs, GlyphSet::Ascii);
    for (name, buf) in populate_states(ascii_theme) {
        let area = *buf.area();
        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buf.cell((x, y)).unwrap();
                for ch in cell.symbol().chars() {
                    assert!(ch.is_ascii(), "{name} ({x},{y}) non-ASCII glyph {ch:?}");
                }
            }
        }
    }
}

#[test]
fn all_builtin_themes_preserve_non_color_cues_in_every_state() {
    for id in ThemeId::ALL {
        let theme = for_id(id);
        let frames = populate_states(theme);
        let sel_text = frame_text(&frames[0].1);
        assert!(sel_text.contains("[A]"), "{} missing [A]", id.as_str());
        assert!(sel_text.contains("[  3]"), "{} missing [  3]", id.as_str());
        assert!(sel_text.contains("A1:A3"), "{} missing A1:A3", id.as_str());
        assert!(sel_text.contains("[EXT]"), "{} missing [EXT]", id.as_str());
        assert!(
            sel_text.contains("#DIV/0!"),
            "{} missing #DIV/0!",
            id.as_str()
        );

        let menu_text = frame_text(&frames[1].1);
        assert!(menu_text.contains(">File"), "{} missing >File", id.as_str());
        assert!(
            menu_text.contains("MENU"),
            "{} missing MENU badge",
            id.as_str()
        );

        let dialog_text = frame_text(&frames[2].1);
        assert!(
            dialog_text.contains("DIALOG"),
            "{} missing DIALOG badge",
            id.as_str()
        );
        assert!(dialog_text.contains("[OK]"), "{} missing [OK]", id.as_str());
        assert!(
            dialog_text.contains("[Cancel]"),
            "{} missing [Cancel]",
            id.as_str()
        );
    }
}
