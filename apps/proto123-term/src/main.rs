//! Native terminal front end: crossterm event loop around the shared core. No spreadsheet
//! rules live here; events are translated, mapped by `InputMapper`, and dispatched.

use crossterm::event;
use proto123_engine::{Command, Mode, Redraw, Session};
use proto123_term::args::{CliAction, USAGE, parse_args};
use proto123_term::guard::TerminalGuard;
use proto123_term::translate;
use proto123_theme::{CapabilityProfile, GlyphSet, ThemeId};
use proto123_tui::{InputMapper, Key, LayoutMap, RawKind, render};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, stdout};
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let action = match parse_args(raw_args.iter().map(String::as_str)) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("proto123-term: {msg}");
            return ExitCode::from(2);
        }
    };
    let (theme_override, glyphs_override) = match action {
        CliAction::Help => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        CliAction::Version => {
            println!("proto123-term {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        CliAction::Run { theme, glyphs } => (theme, glyphs),
    };

    TerminalGuard::install_panic_hook();
    match run(theme_override, glyphs_override) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Printed only after the guard has restored the terminal (SC-014).
            eprintln!("proto123-term: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(theme_override: Option<ThemeId>, glyphs_override: Option<GlyphSet>) -> io::Result<()> {
    let start = Instant::now();
    let term_env = std::env::var("TERM").ok();
    let colorterm_env = std::env::var("COLORTERM").ok();
    let profile = CapabilityProfile::detect(term_env.as_deref(), colorterm_env.as_deref());
    let mut session = Session::new(profile.theme());
    if theme_override.is_some() || glyphs_override.is_some() {
        let id = theme_override.unwrap_or_else(|| session.theme().id);
        let glyphs = glyphs_override.or(Some(session.theme().glyphs));
        session.dispatch(Command::SessionSetTheme { id, glyphs });
    }
    let mut mapper = InputMapper::new(session.theme().metrics);
    let mut layout = LayoutMap::default();
    {
        let guard = TerminalGuard::enter()?;
        let result = event_loop(&mut session, &mut mapper, &mut layout, start);
        drop(guard);
        result?;
    }
    Ok(())
}

fn event_loop(
    session: &mut Session,
    mapper: &mut InputMapper,
    layout: &mut LayoutMap,
    start: Instant,
) -> io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let size = terminal.size()?;
    let mut pending = vec![RawKind::Resize {
        cols: size.width,
        rows: size.height,
    }];
    loop {
        for kind in pending.drain(..) {
            let t_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            let raw = proto123_tui::RawEvent { t_ms, kind };
            let mut redraw = Redraw::None;
            for command in mapper.map(raw, layout, session) {
                redraw = combine(redraw, session.dispatch(command).redraw);
            }
            if matches!(raw.kind, RawKind::Resize { .. }) {
                redraw = Redraw::Full;
            }
            if redraw == Redraw::Full {
                terminal.clear()?;
            }
            if redraw != Redraw::None {
                draw(&mut terminal, session, layout)?;
            }
        }
        let ev = event::read()?;
        let t_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let Some(raw) = translate::translate(ev, t_ms) else {
            continue;
        };
        if is_quit(&raw, session) {
            return Ok(());
        }
        pending.push(raw.kind);
    }
}

/// `Ctrl+Q` quits from READY (contracts/native-cli.md); the core never sees it.
fn is_quit(raw: &proto123_tui::RawEvent, session: &Session) -> bool {
    matches!(
        raw.kind,
        RawKind::Key { key: Key::Char('q'), mods } if mods.ctrl
    ) && session.mode() == Mode::Ready
}

fn combine(a: Redraw, b: Redraw) -> Redraw {
    match (a, b) {
        (Redraw::Full, _) | (_, Redraw::Full) => Redraw::Full,
        (Redraw::Regions, _) | (_, Redraw::Regions) => Redraw::Regions,
        _ => Redraw::None,
    }
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &Session,
    layout: &mut LayoutMap,
) -> io::Result<()> {
    let mut next = LayoutMap::default();
    terminal.draw(|frame| {
        next = render(session, frame.area(), frame.buffer_mut());
    })?;
    *layout = next;
    Ok(())
}
