# Quickstart: Validate the Foundational MVP (Spec 001)

A run guide proving the slice works end-to-end. Contracts: [commands](contracts/commands.md), [raw input](contracts/raw-input.md), [wasm bridge](contracts/wasm-bridge.md), [native CLI](contracts/native-cli.md), [formula grammar](contracts/formula-grammar.md), [xtask](contracts/xtask.md). Entities: [data-model.md](data-model.md).

## Prerequisites

- `rustup` (the pinned 1.99.1 toolchain and `wasm32-unknown-unknown` install automatically from `rust-toolchain.toml`).
- `cargo install --locked trunk --version 0.22.2` (Trunk fetches `wasm-bindgen` and `wasm-opt` itself).
- Docker (only for the container check).

## 1. Guardrails (US7, SC-011)

```sh
cargo xtask layers          # expect: exit 0
cargo xtask menu-commands   # expect: exit 0
cargo test -p xtask         # seeded violations: upward edge, unregistered crate, testkit as normal dep,
                            # ratatui in L4, unregistered menu command, duplicate trigger → each reported
cargo test -p proto123-engine --test it menu   # verify() catches seeded bad catalogs
```

Manual seed: add `proto123-engine` as a dependency of `proto123-coord`, run `cargo xtask layers` → exit 1 naming `proto123-coord (L0) -> proto123-engine (L4)`; revert.

## 2. Core correctness (US1–US3, SC-003–SC-006)

```sh
cargo test -p proto123-coord -p proto123-value -p proto123-formula -p proto123-model -p proto123-calc -p proto123-ops
```

Must include exact-value tests for: `=2+3*4`→14, `=(2+3)*4`→20, `=-2^2`→4, `=2^3^2`→64, `=10/4`→2.5, `=0.1+0.2`→`0.3`, `=1-0.9`→`0.1`; `#DIV/0!`, `#NAME?` (text kept), `#VALUE!`, `#REF!`, `#CIRC!` (self, 2-cycle, 500-cycle, range cycle) and recovery; aggregates over nothing; `=SUM(A1:XFD1048576)` over 10 populated cells; 10,000-deep chain; display steps 1–4; copy tiling (`B1`→`B2:B5`, 1×3 into `B1:B10` fills `B1:B9`); `$` handling; move redirects and `#REF!`; overlapping copy/move; 100-step undo/redo round-trip equality; SC-003 (`RecalcReport.evaluated.len() == 10`).

## 3. Session, layout, and input (US1–US6, US8 frame rules)

```sh
cargo test -p proto123-engine          # modes, commands, prompts, diagnostics, live-region text
cargo test -p proto123-tui             # LayoutMap, pixel_to_cell (DPR 1/1.25/2), InputMapper gestures,
                                       # console-profile frames: ANSI-16 only, allowed glyphs, no bold
cargo test -p proto123-theme           # contrast ≥ 4.5 / ≥ 3.0 every theme; CapabilityProfile::detect table
cargo test -p proto123-engine --test panic_hunt -- --ignored   # hostile input sweep: zero panics/hangs (SC-005)
```

## 4. Both targets run the same scenarios (FR-049, SC-012)

```sh
cargo test -p proto123-wasm   # runs natively: US1–US5 scenarios through DOM-shaped translate()
cargo test -p proto123-term   # same scenarios through crossterm translate(); identical snapshots
```

## 5. Native terminal (US8, SC-013, SC-014)

```sh
cargo run --release -p proto123-term
```

Expected: empty grid, `[A1]` and `READY` on the control line, root bar `File  Edit  Range  View`.
Smoke (keyboard only): `120⏎ 80⏎`, go to `A3`, `=SUM(A1:A2)⏎` → `200`; change `A1` to `150` → `230`; `F8 ↓ ↓` → `[EXT]`; `/ R E` erases; `Ctrl+Z` restores; `/ V T` cycles theme; `Ctrl+Q` quits and the terminal is normal (prompt, echo, cursor, no mouse escape garbage).
Also check:
- `TERM=linux cargo run -p proto123-term` → console16 theme, single-line borders. `TERM=dumb` → ASCII glyphs. `--glyphs ascii` overrides.
- Resize below 80×24 → "enlarge the window" message; keys do nothing; resize back restores.
- Mouse (desktop terminal / `tmux` with `mouse on`): click, double-click edit, drag `B2:D6`, wheel, hover menu, click `Edit → Undo`.
- Internal-error path: run the test `cargo test -p proto123-term --test it guard_restores_on_panic` (drives the guard + panic hook against a fake writer and asserts the restore sequence).
- Manual release check in a Linux virtual console (`Ctrl+Alt+F3`, log in, run the binary): edit, `/` menu, `F8` range selection, undo, quit → console usable.

## 6. Web target (US4, US6, US7, SC-009)

```sh
trunk serve --open            # http://127.0.0.1:8080
```

- Repeat the §5 smoke with keyboard, then with mouse only (SC-001: < 60 s).
- DevTools Elements: `<body>` holds exactly one `<canvas>` and one `[aria-live]` element; count unchanged after scrolling to `XFD1048576` and back.
- Click `C5`, `Shift+click` `E9`; double-click edits; drag beyond the bottom edge auto-scrolls; wheel = 3 rows; `Shift+wheel` scrolls columns; click in the formula bar positions the caret; type `=SUM(` then drag `B2:B9` → buffer `=SUM(B2:B9`, mode `POINT`.
- Zoom the browser to 125 % and 200 % → clicks still land on the cell under the pointer.
- Screen reader (or watch `#live` in DevTools): moving to `C4` announces `C4, formula =SUM(C1:C3), value 150, READY`.
- Performance (SC-002): this slice has no file loading, so the 10,000-cell reference sheet is measured natively with `cargo run --release -p proto123-tui --example frame_budget` (p50/p99 input→frame report). In the browser, record a DevTools Performance trace while holding an arrow key and dragging on a hand-filled sheet; each input task must finish in < 16 ms.
- Offline: after load, set DevTools Network to *Offline*; the app keeps working.

## 7. Release build, size, container (US7, FR-041, FR-042)

```sh
cargo xtask wasm              # prints: wasm: dist/…_bg.wasm raw=<n> gzip=<n>   (record as baseline)
cargo build -p proto123-term  # CI also builds on macOS and Windows
cargo xtask docker-check      # builds image, asserts Content-Type: application/wasm
docker run --rm -p 8080:80 proto123:check   # open http://localhost:8080, app works with no backend
```

## 8. Full gate before PR

```sh
cargo xtask ci
```

Report in the PR which of the above ran and passed (AGENTS rule 34); the manual Linux-console and screen-reader checks are reported as manual.
