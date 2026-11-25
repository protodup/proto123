//! Reference-workload frame budget measurement (SC-002, research R14).
//! Builds a 10,000-cell / 5,000-formula sheet and reports p50/p99 input->buffer-ready time
//! for arrow, click, drag, and wheel events using `ratatui::backend::TestBackend`.

use proto123_coord::CellAddr;
use proto123_engine::{Command, Redraw, Session, Then};
use proto123_theme::high_contrast;
use proto123_tui::{Button, InputMapper, Key, LayoutMap, Mods, Pos, RawEvent, RawKind, render};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::time::{Duration, Instant};

fn build_reference_session() -> Session {
    let mut s = Session::new(high_contrast());
    for row in 0..5_000u32 {
        if let Some(a) = CellAddr::new(0, row) {
            s.dispatch(Command::NavSelectCell(a));
            s.dispatch(Command::CellBeginEntry('1'));
            s.dispatch(Command::CellCommitEdit(Then::Stay));
        }
        if let Some(b) = CellAddr::new(1, row) {
            s.dispatch(Command::NavSelectCell(b));
            s.dispatch(Command::CellBeginEntry('='));
            for c in format!("A{}*2+1", row + 1).chars() {
                s.dispatch(Command::CellEditInput(c));
            }
            s.dispatch(Command::CellCommitEdit(Then::Stay));
        }
    }
    s.dispatch(Command::NavSelectCell(CellAddr::ORIGIN));
    s
}

fn measure_kind(
    name: &str,
    session: &mut Session,
    mapper: &mut InputMapper,
    terminal: &mut Terminal<TestBackend>,
    layout: &mut LayoutMap,
    events: &[RawKind],
) {
    let mut samples: Vec<Duration> = Vec::with_capacity(events.len());
    for (idx, kind) in events.iter().enumerate() {
        let start = Instant::now();
        let raw = RawEvent {
            t_ms: u64::try_from(idx + 1).unwrap_or(1) * 16,
            kind: *kind,
        };
        let mut redraw = Redraw::None;
        for cmd in mapper.map(raw, layout, session) {
            let out = session.dispatch(cmd);
            if out.redraw == Redraw::Full {
                redraw = Redraw::Full;
            } else if out.redraw == Redraw::Regions && redraw == Redraw::None {
                redraw = Redraw::Regions;
            }
        }
        if redraw == Redraw::Full {
            let _ = terminal.clear();
        }
        if redraw != Redraw::None {
            let mut next = LayoutMap::default();
            let _ = terminal.draw(|f| {
                next = render(session, f.area(), f.buffer_mut());
            });
            *layout = next;
        }
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    let len = samples.len();
    let p50 = samples.get(len / 2).copied().unwrap_or_default();
    let p99 = samples
        .get((len * 99) / 100)
        .or_else(|| samples.last())
        .copied()
        .unwrap_or_default();
    println!(
        "frame_budget {name:<8} n={len:<4} p50={:.3}ms p99={:.3}ms",
        p50.as_secs_f64() * 1_000.0,
        p99.as_secs_f64() * 1_000.0
    );
}

fn main() {
    let mut session = build_reference_session();
    let mut mapper = InputMapper::new(session.theme().metrics);
    let Ok(mut terminal) = Terminal::new(TestBackend::new(80, 24));
    let mut layout = LayoutMap::default();
    for cmd in mapper.map(
        RawEvent {
            t_ms: 0,
            kind: RawKind::Resize { cols: 80, rows: 24 },
        },
        &layout,
        &session,
    ) {
        session.dispatch(cmd);
    }
    let _ = terminal.draw(|f| {
        layout = render(&session, f.area(), f.buffer_mut());
    });

    println!(
        "reference sheet populated cells: {}",
        session.calc().sheet().populated()
    );

    let arrows: Vec<RawKind> = (0..200)
        .map(|i| RawKind::Key {
            key: if i % 2 == 0 { Key::Down } else { Key::Right },
            mods: Mods::default(),
        })
        .collect();
    measure_kind(
        "arrow",
        &mut session,
        &mut mapper,
        &mut terminal,
        &mut layout,
        &arrows,
    );

    let clicks: Vec<RawKind> = (0..200)
        .flat_map(|i| {
            let col = 6 + ((i as u16 * 9) % 60);
            let row = 4 + (i as u16 % 15);
            [
                RawKind::PointerDown {
                    at: Pos::Cell { col, row },
                    button: Button::Left,
                    mods: Mods::default(),
                },
                RawKind::PointerUp {
                    at: Pos::Cell { col, row },
                    button: Button::Left,
                    mods: Mods::default(),
                },
            ]
        })
        .collect();
    measure_kind(
        "click",
        &mut session,
        &mut mapper,
        &mut terminal,
        &mut layout,
        &clicks,
    );

    let mut drags = vec![RawKind::PointerDown {
        at: Pos::Cell { col: 6, row: 4 },
        button: Button::Left,
        mods: Mods::default(),
    }];
    for i in 0..200u16 {
        drags.push(RawKind::PointerMove {
            at: Pos::Cell {
                col: 6 + (i % 5) * 9,
                row: 4 + (i % 15),
            },
            buttons_down: true,
            mods: Mods::default(),
        });
    }
    drags.push(RawKind::PointerUp {
        at: Pos::Cell { col: 15, row: 10 },
        button: Button::Left,
        mods: Mods::default(),
    });
    measure_kind(
        "drag",
        &mut session,
        &mut mapper,
        &mut terminal,
        &mut layout,
        &drags,
    );

    let wheels: Vec<RawKind> = (0..200)
        .map(|i| RawKind::Wheel {
            dx: 0,
            dy: if i % 2 == 0 { 1 } else { -1 },
            mods: Mods::default(),
        })
        .collect();
    measure_kind(
        "wheel",
        &mut session,
        &mut mapper,
        &mut terminal,
        &mut layout,
        &wheels,
    );
}
