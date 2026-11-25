//! Native terminal front end: crossterm event loop around the shared core. No spreadsheet
//! rules live here; events are translated, mapped by `InputMapper`, and dispatched.

mod guard;
mod translate;

use crossterm::event;
use guard::TerminalGuard;
use proto123_engine::Session;
use proto123_theme::{GridMetrics, high_contrast};
use proto123_engine::Redraw;
use proto123_tui::{InputMapper, Key, LayoutMap, RawKind, render};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, stdout};
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    TerminalGuard::install_panic_hook();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Printed only after the guard has restored the terminal (see `run`).
            eprintln!("proto123-term: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> io::Result<()> {
    let start = Instant::now();
    let mut session = Session::new(high_contrast());
    let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
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
    let mut pending = vec![RawKind::Resize { cols: size.width, rows: size.height }];
    loop {
        for kind in pending.drain(..) {
            let t_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            let raw = proto123_tui::RawEvent { t_ms, kind };
            let mut redraw = Redraw::None;
            for command in mapper.map(raw, layout, session) {
                redraw = combine(redraw, session.dispatch(command).redraw);
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

/// `Ctrl+Q` quits from READY (contracts/commands.md); the core never sees it.
fn is_quit(raw: &proto123_tui::RawEvent, session: &Session) -> bool {
    matches!(
        raw.kind,
        RawKind::Key { key: Key::Char('q'), mods } if mods.ctrl
    ) && session.mode() == proto123_engine::Mode::Ready
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
