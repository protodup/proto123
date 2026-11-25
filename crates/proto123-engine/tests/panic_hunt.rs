//! Hostile-input sweep (SC-005, AGENTS rule 30). Ignored by default; run with
//! `cargo test -p proto123-engine --test panic_hunt -- --ignored`.
//! Every input is driven through `InputMapper` and `Session::dispatch`; the test fails only
//! by panicking, and each run is bounded in steps.

#[cfg(test)]
mod hunt {
    use proto123_coord::{CellAddr, MAX_COL, MAX_ROW};
    use proto123_engine::{CaretKey, Command, HomeKind, Session, Then};
    use proto123_theme::{GridMetrics, high_contrast};
    use proto123_tui::{InputMapper, Key, LayoutMap, Mods, RawEvent, RawKind, render};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    /// Small deterministic generator, so a failing seed reproduces exactly.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }

        fn pick<T: Copy>(&mut self, items: &[T]) -> T {
            let idx = usize::try_from(self.next()).unwrap_or(0) % items.len();
            items[idx]
        }
    }

    const CHARS: &[char] = &[
        '1', 'a', 'Z', '=', '(', ')', '#', '\'', '"', '^', ' ', '+', '-', '/', '*', '.', ',', ';', ':', '$', '@', 'é', '\u{0}',
        '\u{1F600}', 'E', 'F', 'T',
    ];
    const NAMED: &[Key] = &[
        Key::Enter, Key::Esc, Key::Tab, Key::BackTab, Key::Backspace, Key::Delete, Key::Left, Key::Right, Key::Up,
        Key::Down, Key::Home, Key::End, Key::PageUp, Key::PageDown, Key::F(2), Key::Other,
    ];
    const SIZES: &[(u16, u16)] = &[(80, 24), (79, 24), (10, 3), (0, 0), (300, 100), (u16::MAX, u16::MAX)];

    fn random_event(rng: &mut Lcg, t_ms: u64) -> RawEvent {
        let kind = match rng.next() % 3 {
            0 => {
                let c = rng.pick(CHARS);
                RawKind::Key { key: Key::Char(c), mods: Mods::default() }
            }
            1 => {
                let key = rng.pick(NAMED);
                let mods = Mods { shift: rng.next().is_multiple_of(2), ctrl: rng.next().is_multiple_of(4), alt: rng.next().is_multiple_of(5), meta: false };
                RawKind::Key { key, mods }
            }
            _ => {
                let (cols, rows) = rng.pick(SIZES);
                RawKind::Resize { cols, rows }
            }
        };
        RawEvent { t_ms, kind }
    }

    /// Runs `steps` random events, then checks that the session still answers a frame.
    fn sweep(seed: u64, steps: usize) {
        let mut rng = Lcg(seed);
        let mut session = Session::new(high_contrast());
        let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
        let area = Rect::new(0, 0, 80, 24);
        let mut layout = LayoutMap::default();
        for step in 0..steps {
            let event = random_event(&mut rng, step as u64);
            for command in mapper.map(event, &layout, &session) {
                session.dispatch(command);
            }
            let mut buf = Buffer::empty(area);
            layout = render(&session, area, &mut buf);
        }
    }

    /// Typed hostile texts: deep nesting, very long formulas, and grid-edge references.
    fn typed(texts: &[String]) {
        let mut session = Session::new(high_contrast());
        let mut mapper = InputMapper::new(GridMetrics::DEFAULT);
        let area = Rect::new(0, 0, 80, 24);
        let mut layout = LayoutMap::default();
        let mut t = 0;
        let mut feed = |session: &mut Session, layout: &mut LayoutMap, kind: RawKind| {
            t += 1;
            for command in mapper.map(RawEvent { t_ms: t, kind }, layout, session) {
                session.dispatch(command);
            }
            let mut buf = Buffer::empty(area);
            *layout = render(session, area, &mut buf);
        };
        feed(&mut session, &mut layout, RawKind::Resize { cols: 80, rows: 24 });
        for text in texts {
            for c in text.chars() {
                feed(&mut session, &mut layout, RawKind::Key { key: Key::Char(c), mods: Mods::default() });
            }
            feed(&mut session, &mut layout, RawKind::Key { key: Key::Enter, mods: Mods::default() });
        }
    }

    #[test]
    #[ignore = "hostile-input sweep; run with --ignored"]
    fn random_key_and_resize_sequences_never_panic() {
        for seed in 1..=4u64 {
            sweep(seed, 2_000);
        }
    }

    #[test]
    #[ignore = "hostile-input sweep; run with --ignored"]
    fn deep_nesting_and_long_formulas_never_panic() {
        let deep = format!("={}1{}", "(".repeat(300), ")".repeat(300));
        let long = format!("={}", "1+".repeat(4_000));
        let runaway = "=".to_owned() + &"-".repeat(10_000) + "1";
        let cycles = (0..20).map(|i| format!("=A{}", i + 2)).collect::<Vec<_>>();
        typed(&[deep, long, runaway]);
        typed(&cycles);
    }

    #[test]
    #[ignore = "hostile-input sweep; run with --ignored"]
    fn grid_limit_commands_never_panic() {
        let mut s = Session::new(high_contrast());
        let far = CellAddr::new(MAX_COL, MAX_ROW).unwrap_or(CellAddr::ORIGIN);
        let commands = [
            Command::NavSelectCell(far),
            Command::NavMoveRelative { dcol: 1, drow: 1 },
            Command::NavMoveRelative { dcol: i64::MAX, drow: i64::MIN },
            Command::NavMoveRelative { dcol: i64::MIN, drow: i64::MAX },
            Command::NavPageScroll { horizontal: true, dir: i64::MAX },
            Command::NavPageScroll { horizontal: false, dir: i64::MIN },
            Command::NavJumpHome(HomeKind::RowStart),
            Command::ViewScroll { dcol: i64::MAX, drow: i64::MAX },
            Command::ViewScroll { dcol: i64::MIN, drow: i64::MIN },
            Command::SessionResize { cols: u32::MAX, rows: u32::MAX },
            Command::SessionResize { cols: 0, rows: 0 },
            Command::CellEditCaret(CaretKey::Delete),
            Command::CellCommitEdit(Then::Move { dcol: i64::MAX, drow: i64::MAX }),
        ];
        for command in commands {
            s.dispatch(command);
        }
        assert!(s.active().row() <= MAX_ROW);
    }
}
