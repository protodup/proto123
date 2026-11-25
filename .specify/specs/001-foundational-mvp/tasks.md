---

description: "Task list for Spec 001 — Foundational MVP: Calculating Grid on a Single Canvas"
---

# Tasks: Foundational MVP — Calculating Grid on a Single Canvas

**Input**: Design documents from `.specify/specs/001-foundational-mvp/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Requested. The spec defines an Independent Test per story, a shared scenario suite run on both targets (FR-049, SC-012), seeded-violation checks (SC-011), exact-value formula checks (SC-004), and a hostile-input sweep (SC-005). Test tasks come before the implementation they guard inside each story. Per AGENTS rule 31, each crate has one integration binary `tests/it/main.rs` with one module file per area (`tests/it/<area>.rs`); the only extra binaries are `crates/proto123-engine/tests/panic_hunt.rs` (named by AGENTS.md) and `crates/proto123-tui/examples/frame_budget.rs`.

**Organization**: Tasks are grouped by user story so each story is an independently testable increment on top of Phases 1–2.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: User story the task belongs to (US1–US8)
- Field constraints are cited as `data-model.md` `Entity.field`; the constraint text lives there and is not repeated here.

## Path Conventions

Cargo workspace at the repository root: libraries in `crates/<name>/src/`, apps in `apps/<name>/src/`, web host in `web/`, tooling in `xtask/src/`. Every crate is `src/lib.rs` (or `src/main.rs`) + `tests/it/main.rs`, and its `Cargo.toml` has `[lints] workspace = true`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace skeleton, toolchain, lint policy, and empty crates that compile on both targets.

- [X] T001 Create root `Cargo.toml` with `[workspace]` members (`crates/proto123-coord`, `-value`, `-theme`, `-formula`, `-model`, `-calc`, `-ops`, `-engine`, `-tui`, `-testkit`, `apps/proto123-wasm`, `apps/proto123-term`, `xtask`), `resolver = "3"`, `[workspace.package]` (edition 2024, `rust-version = "1.99"`, license), `[workspace.dependencies]` pinning `ratatui` 0.30 (`default-features = false`), `unicode-width` 0.2, `ratzilla` 0.3, `wasm-bindgen` 0.2, `crossterm` 0.29, `cargo_metadata`, `flate2`, `[workspace.lints.rust] unsafe_code = "forbid"`, `[workspace.lints.clippy]` denying `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` (allowed in tests via `clippy.toml`, not via `#[allow]`), and profiles `release` (`panic = "unwind"`) and `release-wasm` (inherits release, `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`) per research R1, R4, R13
- [X] T002 [P] Create `rust-toolchain.toml` pinning 1.99.1 with `targets = ["wasm32-unknown-unknown"]` and `components = ["clippy", "rustfmt"]`, and `clippy.toml` with `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests = true`
- [X] T003 [P] Create `.cargo/config.toml` with alias `xtask = "run -q -p xtask --"` (contracts/xtask.md)
- [X] T004 [P] Update `.gitignore`: stop ignoring `Cargo.lock` (the workspace ships binaries), add `dist/`, `log/`, `plan/`
- [X] T005 Create the ten library crates with `Cargo.toml` (`[lints] workspace = true`, no dependencies yet), empty `src/lib.rs`, and `tests/it/main.rs`: `crates/proto123-coord`, `crates/proto123-value`, `crates/proto123-theme`, `crates/proto123-formula`, `crates/proto123-model`, `crates/proto123-calc`, `crates/proto123-ops`, `crates/proto123-engine`, `crates/proto123-tui`, `crates/proto123-testkit`
- [X] T006 Create the two app crates and `xtask` with `Cargo.toml` (`[lints] workspace = true`) and placeholder entry points: `apps/proto123-wasm/src/lib.rs` (`crate-type = ["cdylib", "rlib"]`; `pub mod translate;` always compiled, `mod bridge;` behind `#[cfg(target_arch = "wasm32")]` so the crate's tests build and run natively), `apps/proto123-term/src/main.rs`, `xtask/src/main.rs` (subcommand dispatch printing usage, exit 2 on unknown subcommand per contracts/xtask.md)
- [X] T007 Wire intra-workspace dependencies per plan.md Structure Decision: value→coord; formula→coord,value; model→coord,value,formula; calc→model,formula; ops→calc,model,formula; engine→ops,calc,model,formula,value,coord,theme; tui→engine,theme,coord,value + `ratatui`, `unicode-width`; testkit→L0–L5; wasm→tui,engine,theme, plus `ratzilla` and `wasm-bindgen` only under `[target.'cfg(target_arch = "wasm32")'.dependencies]` (native `cargo build --workspace` and `cargo test -p proto123-wasm` then never compile Ratzilla or `beamterm-renderer`); term→tui,engine,theme + `ratatui` (`crossterm` feature), `crossterm`; every crate's `[dev-dependencies]` may add `proto123-testkit`. Verify with `cargo build --workspace` and `cargo build --target wasm32-unknown-unknown -p proto123-coord -p proto123-value -p proto123-theme -p proto123-formula -p proto123-model -p proto123-calc -p proto123-ops -p proto123-engine -p proto123-tui`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Primitives, session/command skeleton, layout skeleton, raw-input path, both target shells, the scenario harness, and the layer guardrail. Every story builds on these.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

### Guardrail: layer check (also satisfies US7 scenario 1 early, so every later crate is checked)

- [X] T008 [P] Write seeded-violation tests for `layers::check` in `xtask/tests/it/layers.rs`: upward normal dependency (reports `proto123-coord (L0) -> proto123-engine (L4)`), unregistered workspace member, `proto123-testkit` as a normal dependency, `ratatui`/`crossterm`/`ratzilla`/`web-sys`/`wasm-bindgen` in an L0–L4 crate, a backend crate (`crossterm`, `ratzilla`) in L5, a graphical-session or GPU crate (`x11`, `x11rb`, `x11-clipboard`, `wayland-client`, `wgpu`, `glutin`) anywhere in `proto123-term`'s normal dependency tree (FR-044), and an allowed same-layer edge (`proto123-model -> proto123-formula`) that must pass (research R12, SC-011)
- [X] T009 Implement `xtask/src/layers.rs`: `const LAYERS` for every planned crate (L0–L6, `Support`, `Tooling`; also lists the absent `proto123-ods`, `proto123-xlsx`, `proto123-csv`, `proto123-cli`), pure `check(packages, LAYERS) -> Vec<LayerViolation>` (every rule in research R12, including the graphical-session/GPU crate ban for `proto123-term`), and `cargo xtask layers` in `xtask/src/main.rs` reading `cargo_metadata` and exiting 0/1/2 per contracts/xtask.md

### L0 primitives

- [X] T010 [P] Write tests in `crates/proto123-coord/tests/it/addr.rs`: A1 parse/print round-trip incl. `XFD1048576`, case-insensitive letters printed upper-case, `$A$1`/`$A1`/`A$1` flags, rejection of `XFE1`, row 0, row 1048577, `RefAddr::shifted` keeping absolute parts and returning `None` off-grid, `CellRange` normalization from any corner, `area`, `contains`, row-major `iter`
- [X] T011 [P] Implement `CellAddr` (fallible `new`, `Display`, row-major `Ord`, `Hash`) and grid limits in `crates/proto123-coord/src/addr.rs` per `data-model.md` `CellAddr.col` and `CellAddr.row`
- [X] T012 Implement `RefAddr` parse/print/`shifted` in `crates/proto123-coord/src/refaddr.rs` per `data-model.md` `RefAddr.addr`, `RefAddr.col_abs`, `RefAddr.row_abs`, and `CellRange` (`width`, `height`, `area: u64`, `contains`, `iter`, parse `A1:B3`) in `crates/proto123-coord/src/range.rs` per `data-model.md` `CellRange.start` and `CellRange.end`; re-export from `crates/proto123-coord/src/lib.rs`
- [X] T013 [P] Write tests in `crates/proto123-value/tests/it/display.rs` with the exact FR-006 table: `0.1+0.2`→`0.3`, `1-0.9`→`0.1`, `10/4`→`2.5`, `1/3`→`0.3333333`, `0.000001234`→`0.0000012`, `123456789012`→`1.235E+11`, `0.0000000012`→`1.2E-09`, `-0`→`0`, half-away-from-zero ties, `#` fill when width is below the scientific minimum, `format_full` never above 15 significant digits, error tokens `#DIV/0!` `#REF!` `#NAME?` `#VALUE!` `#CIRC!`, Bool `TRUE`/`FALSE`
- [X] T014 [P] Implement `Value` and `ErrorKind` (with `token()`) in `crates/proto123-value/src/lib.rs` per `data-model.md` `Value.Number` (finite invariant) and `ErrorKind`; implement `format_general(n, width)` and `format_full(n)` in `crates/proto123-value/src/display.rs` with decimal-string rounding per research R7 and contracts/formula-grammar.md §4
- [X] T015 [P] Implement theme types in `crates/proto123-theme/src/lib.rs`: `Color` (`Rgb`, `Ansi16`), `Ansi16`, `GlyphSet` per `data-model.md` `GlyphSet`, `ThemeTokens` per `data-model.md` `ThemeTokens`, `Theme` per `data-model.md` `Theme.id`, `Theme.glyphs`, `Theme.allow_bold`, `Theme.metrics` (`GridMetrics.col_width`, `GridMetrics.min_cols`, `GridMetrics.min_rows`, `GridMetrics.font_px`, `GridMetrics.font_family`, `GridMetrics.cell_px`, `GridMetrics.origin_px`), `ThemeId` per `data-model.md` `ThemeId` with `next()` in cycle order, and the `high-contrast` built-in theme in `crates/proto123-theme/src/builtin.rs` (the other two themes land in US5/US8)

### L1–L2 model shell

- [X] T016 Implement `Alignment` and `CellContent` in `crates/proto123-model/src/content.rs` per `data-model.md` `Alignment` and `CellContent.Label` (prefix kept for the formula bar); in this phase `CellContent::Formula` holds a `proto123_formula::Formula` stub type defined in `crates/proto123-formula/src/lib.rs` (`expr`, `canonical`) per `data-model.md` `Formula.canonical`
- [X] T017 Implement `Cell`, `Sheet`, `Workbook` in `crates/proto123-model/src/sheet.rs` per `data-model.md` `Cell.value`, `Sheet.cells` (sparsity: storing Empty removes the entry), and `Workbook.sheet` (exactly one), with `is_empty()`; add a test in `crates/proto123-model/tests/it/sheet.rs` that writing then clearing `XFD1048576` leaves the map empty (FR-001, far-corner edge case)

### L4 engine skeleton

- [X] T018 [P] Create the central English string catalog in `crates/proto123-engine/src/strings.rs`: `StringKey` enum + `fn text(StringKey) -> &'static str` holding mode badges (`READY`, `VALUE`, `LABEL`, `EDIT`, `POINT`, `MENU`, `DIALOG` per `docs/feature-names.md` §3); menu label/description, status, and diagnostic keys are added by the story that first uses them, never ahead of time (FR-027, research R11)
- [ ] T018a [P] Update `docs/interaction-spec.md` §2: add the `DIALOG` mode and replace the transition list with `data-model.md` "Mode transitions", so the document FR-017 names is correct before any story implements transitions (§3 and §5 stay in T116)
- [X] T019 Implement `CommandId`, `CommandSpec { id, name, undoable }`, and `REGISTRY` in `crates/proto123-engine/src/command.rs` with the `nav.*`, `view.scroll`, `cell.*`, and `session.resize` ids from contracts/commands.md (other ids are added by the story that implements them — never as stubs, research R11)
- [ ] T020 Implement `Mode` and `Session` in `crates/proto123-engine/src/session.rs` per `data-model.md` `Session.mode` (exactly one), `Session.active`, `Session.anchor`, `Session.viewport`, `Session.status`, `Session.theme`, `Session.announce`; `Session::new(profile_theme, glyphs)` starts at `A1` in `READY`; `Session::dispatch(Command) -> Outcome { redraw, announce }` as an exhaustive `match` per `data-model.md` `Command`
- [X] T021 Implement `nav.moveRelative`, `nav.selectCell`, `nav.pageScroll`, `nav.jumpHome`, `view.scroll`, `session.resize` and viewport-follow (active cell always visible) in `crates/proto123-engine/src/nav.rs`, clamped to grid limits (US1 sc.9, `docs/interaction-spec.md` §3)
- [X] T022 [P] Write navigation tests in `crates/proto123-engine/tests/it/nav.rs`: arrows, `Tab`/`Shift+Tab`, `Enter`/`Shift+Enter` in READY, `PgUp`/`PgDn`, `Alt+PgUp`/`Alt+PgDn`, `Home`, `Ctrl+Home`, clamping at `A1` and `XFD1048576`, viewport follows active cell

### L5 layout and input skeleton

- [X] T023 [P] Implement `RawEvent`, `RawKind`, `Pos`, `Mods`, `Key`, `Button` in `crates/proto123-tui/src/raw.rs` exactly as contracts/raw-input.md, and the `CellMetrics` struct per `data-model.md` `CellMetrics` (all fields; `from_theme(&GridMetrics, dpr)`, `reconcile(backend_cell_css)`, and `grid_size(css_w, css_h) -> (cols, rows)`) in `crates/proto123-tui/src/pixel.rs`, needed by T031 and T033 (`pixel_to_cell` lands in US4, T090)
- [ ] T024 [P] Implement theme→ratatui mapping in `crates/proto123-tui/src/style.rs` (`Ansi16` → named ratatui colors, `Rgb` → `Color::Rgb`, no `BOLD`/`UNDERLINED` when `allow_bold` is false) per research R6
- [X] T025 Implement `HitTarget` and `LayoutMap { hit(col,row) }` in `crates/proto123-tui/src/layout_map.rs` per `data-model.md` `HitTarget` and `LayoutMap`
- [X] T026 Implement `render(&Session, area, &mut Buffer) -> LayoutMap` in `crates/proto123-tui/src/render.rs` for the 80×24 anatomy of `docs/interaction-spec.md` §1: control line (`[A1]` address box, content, mode badge), always-visible root-bar line placeholder, column/row headers with bracketed active header (`[B]`, `[  3]`), grid with columns `Theme.metrics.col_width` wide (9), number right-alignment via `format_general`, status bar; every color, glyph, and grid metric from theme tokens (no literal widths or sizes in `render.rs`); the same pass records `GridCell`/header/`FormulaBar`/`AddressBox` rects into the `LayoutMap` (FR-031 step 2); below `GridMetrics.min_cols`×`GridMetrics.min_rows` (80×24) draw the "enlarge the window" frame with an empty `LayoutMap`
- [X] T027 Implement `InputMapper::map(RawEvent, &LayoutMap, &Session) -> Vec<Command>` key path in `crates/proto123-tui/src/input.rs` for READY navigation keys and `Resize` (pointer, menu, and edit keys are added by later stories); below 80×24 emit no mutating command (contracts/raw-input.md Guarantees)
- [X] T028 [P] Write layout tests in `crates/proto123-tui/tests/it/render.rs` using ratatui `TestBackend`: fresh session frame shows `[A1]`, `READY`, bracketed `A`/`1` headers; `LayoutMap::hit` returns `GridCell(B3)` for the cell where `B3` is drawn after scrolling; 79×24 shows the enlarge message and an empty map; resize back restores layout; `session.resize` returns `Redraw::Full` (FR-033)
- [X] T028a [P] Write metrics tests in `crates/proto123-tui/tests/it/metrics.rs`: `CellMetrics::from_theme` copies `GridMetrics.cell_px` and `GridMetrics.origin_px` exactly and keeps the given DPR; `reconcile` with an equal backend size returns the theme values and no mismatch; with a different size (for example 9×18) it returns the backend values and the mismatch flag; `grid_size` for 800×480 CSS px at 10×19 with a 5 px origin gives the exact cols/rows (FR-034, FR-043)

### Scenario harness and target shells

- [X] T029 Implement the scenario DSL in `crates/proto123-testkit/src/scenario.rs`: `Scenario { start_grid, events: Vec<RawEvent>, expect: SessionSnapshot }`, `SessionSnapshot` (mode, active, selection, buffer/caret, status key, grid raw + display), `run_core(&Scenario)` (feeds `RawEvent`s straight to `InputMapper` + `Session`), and a `TargetTranslate` trait each app implements to replay the same events through its own translation (research R14, FR-049)
- [X] T030 [P] Implement `apps/proto123-term/src/translate.rs`: crossterm `Event` → `RawEvent` for keys (`Press`/`Repeat` only) and `Resize`, with `t_ms` supplied by the caller (contracts/raw-input.md native table; mouse rows added in US4)
- [X] T031 [P] Implement `apps/proto123-wasm/src/translate.rs` taking DOM-shaped primitives only (no `web-sys` types): `keydown` `(key, code, mods, t_ms)` → `RawEvent::Key` incl. `Shift+Tab` → `BackTab`, and `(css_w, css_h, dpr)` → `Resize` using `CellMetrics` (contracts/raw-input.md web table; pointer and wheel rows added in US4)
- [X] T032 Implement `TerminalGuard` in `apps/proto123-term/src/guard.rs` (raw mode, alternate screen, hidden cursor; `Drop` restores in reverse order) and the event loop in `apps/proto123-term/src/main.rs` (`event::read` → `translate` → `InputMapper` → `dispatch` → `terminal.draw`, skipped when `Outcome.redraw` is `None` and preceded by `terminal.clear()` when it is `Full`, FR-033; `Ctrl+Q` in READY quits; `main` returns `ExitCode` after the guard drops) per contracts/native-cli.md
- [ ] T033 Implement `apps/proto123-wasm/src/bridge.rs` exports `start(canvas_id)`, `on_key`, `on_resize` (contracts/wasm-bridge.md) using Ratzilla `CanvasBackend` for drawing only (draw skipped on `Redraw::None`, full repaint on `Redraw::Full`, FR-033), and build `CellMetrics` (T023) with `from_theme` from the session theme's `GridMetrics`, reconciled against Ratzilla's `cell_size_css_px()` at start and on every resize; a mismatch sets the error status `diag.fontMetricsMismatch` (add it to `strings.rs`) and hit-testing uses the backend's size (research R2, R3, FR-034); verify `cargo build --target wasm32-unknown-unknown -p proto123-wasm` passes under `unsafe_code = "forbid"` — if it does not, stop and ask the operator (plan Complexity Tracking)
- [ ] T034 [P] Create `web/index.html` (one `<canvas id="grid" tabindex="0">`, one `#live` `aria-live` div, Trunk links with `data-wasm-opt="z"` and `data-cargo-profile="release-wasm"`), `web/style.css` (fullscreen canvas, focus ring, `.sr-only`), `web/host.js` (wait for `TrunkApplicationStarted`, call `start`, forward `keydown` with `preventDefault` and `ResizeObserver`/DPR changes; no other logic), and `Trunk.toml` (`target = "web/index.html"`, `dist = "dist"`) per contracts/wasm-bridge.md and research R13
- [X] T035 Implement both `TargetTranslate` impls (`apps/proto123-term/tests/it/scenarios.rs`, `apps/proto123-wasm/tests/it/scenarios.rs`) that replay every scenario in `proto123-testkit` and assert snapshots equal to `run_core`; seed with one navigation scenario in `crates/proto123-testkit/src/scenarios/nav.rs`

**Checkpoint**: `cargo xtask layers` passes; both apps start, show an empty grid at `A1`/`READY`, and navigate with the keyboard; the scenario harness runs on both translations.

---

## Phase 3: User Story 1 — Build a small calculating model with the keyboard (Priority: P1) 🎯 MVP

**Goal**: Enter numbers, labels (with alignment prefixes) and formulas by keyboard; formulas recalculate live.

**Independent Test**: Fresh session, keyboard only: `120⏎ 80⏎`, `Total` in `B3`, `=SUM(A1:A2)` in `A3` → `200`; change `A1` to `150` → `230`; check badge after each key and the formula bar per cell (spec US1).

### Tests for User Story 1

- [X] T036 [P] [US1] Write parser/printer tests in `crates/proto123-formula/tests/it/parse.rs`: literals, refs, ranges, `SUM`/`sum` names upper-cased, `,` and `;` separators, `+`/`@` canonicalized to `=`, minimal parentheses (`=A1-(B1-C1)` kept), `parse(print(e)) == e` round-trip on a table of formulas (contracts/formula-grammar.md §2–§3)
- [X] T037 [P] [US1] Write evaluator tests in `crates/proto123-formula/tests/it/eval.rs` over an in-memory `CellSource`: `+ - * / ^`, unary, Empty→0, Bool→1/0, `SUM`/`AVERAGE`/`MIN`/`MAX`/`COUNT` over refs, ranges, and direct arguments
- [X] T038 [P] [US1] Write recalculation tests in `crates/proto123-calc/tests/it/recalc.rs`: `A3=SUM(A1:A2)` updates on `A1` change; `RecalcReport.evaluated` contains exactly the changed cell and its transitive dependents, each once, in dependency order; SC-003 (10 of 10,000 formulas affected → `evaluated.len() == 10`); unrelated cells not evaluated
- [X] T039 [P] [US1] Write entry tests in `crates/proto123-engine/tests/it/entry.rs`: US1 scenarios 1–8 (badge `VALUE`/`LABEL`/`EDIT`, buffer + caret, `Enter` commits and moves down, `^`/`"`/`'` alignment with prefix shown only in the formula bar, `F2` edit with caret at end, `Esc` restores, `Right` in VALUE commits and moves, `Left` in EDIT moves caret only), plain-number classification (`5`, `5.`, `.5`, `-12.75`, ` 1.5E3 `→1500), empty buffer commit erases the cell
- [X] T040 [P] [US1] Add the US1 keyboard scenario (spec US1 Independent Test) to `crates/proto123-testkit/src/scenarios/us1.rs` and register it in the suite run by T035
- [X] T040a [P] [US1] Write redraw tests in `crates/proto123-tui/tests/it/redraw.rs` using `Buffer::diff` between consecutive frames: committing `150` in `A1` with `A3=SUM(A1:A2)` returns `Redraw::Regions` and changes only the control line, status bar, `A1`, and `A3` cells; moving the active cell changes only the old and new active cell, their headers, and the control line; a keystroke that changes nothing returns `Redraw::None` (FR-033, research R14; the `Redraw::Full` case for theme and resize is asserted in T077 and T028)
- [X] T040b [P] [US1] Write grid-text tests in `crates/proto123-tui/tests/it/cells.rs` using `TestBackend`: numbers right-aligned; `'`/`"`/`^` labels left/right/centered with the prefix absent from the grid but present in the formula bar; a long label spills over empty right neighbours and is cut at the first non-empty one; a number too wide for 9 characters shows the FR-006 scientific form or a full `#` fill, never truncated digits (FR-005, FR-006, Wide content edge case)

### Implementation for User Story 1

- [X] T041 [P] [US1] Implement the lexer in `crates/proto123-formula/src/lexer.rs` (numbers, refs with `$`, names, `TRUE`/`FALSE` before refs, error literals, operators, `(`, `)`, `,`, `;`, `:`, `@`), positions as char indices
- [X] T042 [US1] Implement `Expr`, `UnOp`, `BinOp` in `crates/proto123-formula/src/ast.rs` per `data-model.md` `Expr` (incl. `Expr.Call.name` upper-cased) and the Pratt parser in `crates/proto123-formula/src/parser.rs` with binding powers from research R8 (`+ -` 10, `* /` 20, `^` 30 left-assoc, prefix 40); out-of-grid refs parse to `ErrorLit(Ref)`; replace the T016 stub with the real `Formula` per `data-model.md` `Formula.canonical`
- [X] T043 [US1] Implement the canonical printer in `crates/proto123-formula/src/print.rs` (contracts/formula-grammar.md §3, numbers via `format_full`) and `Formula::references() -> (Vec<CellAddr>, Vec<CellRange>)` in `crates/proto123-formula/src/refs.rs`
- [X] T044 [US1] Implement the `CellSource` trait and evaluator in `crates/proto123-formula/src/eval.rs` per data-model.md "Evaluation rules" (aggregates iterate only populated cells via `for_each_in_range`)
- [X] T045 [US1] Implement `DependencyGraph` in `crates/proto123-calc/src/graph.rs` per `data-model.md` `DependencyGraph.precedents`, `DependencyGraph.cell_dependents`, `DependencyGraph.range_dependents`
- [X] T046 [US1] Implement `CalcSheet::apply(changes) -> RecalcReport` in `crates/proto123-calc/src/lib.rs` per `data-model.md` `CalcSheet.workbook`, `CalcSheet.graph`, `RecalcReport.evaluated`, `RecalcReport.changed_display`: update precedents, iterative BFS collecting dirty set + edges, Kahn ordering, evaluate in order, `CellSource` impl over the sheet (research R9 steps 1, 2, 4; cycle marking lands in US2)
- [X] T047 [US1] Implement entry classification in `crates/proto123-engine/src/entry.rs` per contracts/formula-grammar.md §1 (plain number, label prefix, formula, text, empty → erase) producing `CellContent`
- [X] T048 [US1] Implement `EditState` per `data-model.md` `Session.edit` (caret is a char index) and the commands `cell.beginEntry`, `cell.beginEdit`, `cell.editInput`, `cell.editCaret`, `cell.commitEdit`, `cell.cancelEdit` in `crates/proto123-engine/src/edit.rs` with the VALUE/LABEL/EDIT transitions of `data-model.md` "Mode transitions" (FR-023 arrow semantics); register the ids in `crates/proto123-engine/src/command.rs`; commits go through `CalcSheet::apply` (undo recording arrives in US3)
- [X] T049 [US1] Render the edit buffer with a visible caret on the control line, the formula-bar content (label with prefix, `format_full` number, canonical formula), label alignment and spill-over to empty right neighbours cut at the first non-empty one in `crates/proto123-tui/src/render.rs` (FR-005, Wide content edge case)
- [X] T050 [US1] Extend `InputMapper` in `crates/proto123-tui/src/input.rs` for printable keys, `F2`, `Enter`, `Tab`, `Esc`, `Backspace`, `Delete`, `Home`, `End`, arrows in VALUE/LABEL/EDIT per FR-023; extend both `translate.rs` files only if a key row is missing
- [X] T051 [US1] Add `strings.rs` keys used by US1 (formula-bar labels, status texts) in `crates/proto123-engine/src/strings.rs`; run `cargo test -p proto123-formula -p proto123-calc -p proto123-engine -p proto123-tui -p proto123-term -p proto123-wasm` green

**Checkpoint**: US1 Independent Test passes in the core and on both target translations; the native app and `trunk serve` show a working calculator grid.

---

## Phase 4: User Story 2 — Trust the results: errors, cycles, and correct math (Priority: P1)

**Goal**: Exact OpenFormula arithmetic, typed error tokens that spread and recover, cycle detection, syntax diagnostics, and no crash or hang on hostile input.

**Independent Test**: Enter the US2 formulas and Edge Case formulas; assert exact tokens/values, spreading, recovery after the fix, and the syntax diagnostic (spec US2).

### Tests for User Story 2

- [X] T052 [P] [US2] Write exact-value tests in `crates/proto123-formula/tests/it/semantics.rs`: `=2+3*4`→14, `=(2+3)*4`→20, `=-2^2`→4, `=2^3^2`→64, `=10/4`→2.5, `=0.1+0.2` displays `0.3`, `=1-0.9` displays `0.1`, `=10/B1` (empty)→`#DIV/0!`, `=SUMM(A1:A3)`→`#NAME?` with canonical text kept, `=A1+1` on text→`#VALUE!`, overflow/NaN→`#VALUE!`, first error left-to-right wins, aggregates over nothing (`SUM`/`MIN`/`MAX`/`COUNT`→0, `AVERAGE`→`#DIV/0!`), mixed ranges skip Text/Empty/Bool, error in range spreads, Text direct argument to an aggregate→`#VALUE!`, `XFE1`→`#REF!`, `TRUE`/`FALSE` literals (SC-004)
- [X] T053 [P] [US2] Write parse-error tests in `crates/proto123-formula/tests/it/errors.rs`: `=SUM(A1:` position and kind `UnclosedParen`, `123 Main St` and `1,000` are syntax errors, > 8,192 chars → `TooLong`, > 256 nesting → `TooDeep` per `data-model.md` `ParseError.kind`, no stack overflow on 100,000 `(` (hostile input)
- [X] T054 [P] [US2] Write cycle tests in `crates/proto123-calc/tests/it/cycles.rs`: self-reference, 2-cycle (US2 sc.4) incl. recovery when one side becomes a constant, 500-cycle, 10,000-cycle, range cycle (`A5=SUM(A1:A10)`), dependents of a cycle show `#CIRC!`, 10,000-deep chain recalculates correctly, `=SUM(A1:XFD1048576)` over 10 populated cells evaluates by visiting only those cells
- [X] T055 [P] [US2] Write diagnostic tests in `crates/proto123-engine/tests/it/diagnostics.rs`: unparsable commit by `Enter`, `Tab`, or arrow stays in the edit mode with caret near the problem and status `diag.syntax` (+ `diag.useQuoteForText` when the entry did not start with `=`, `+`, `@`), `Esc` abandons, `'123 Main St` stores a left label, `1.5E3` stores 1500 (US2 sc.8, sc.10, FR-015)
- [X] T056 [P] [US2] Add US2 scenarios to `crates/proto123-testkit/src/scenarios/us2.rs` (error tokens, cycle + recovery, syntax diagnostic) and register them in the shared suite

### Implementation for User Story 2

- [X] T057 [US2] Complete error semantics in `crates/proto123-formula/src/eval.rs`: non-finite → `#VALUE!`, x/0 → `#DIV/0!`, error spreading, unknown function → `#NAME?`, aggregate rules for direct vs. range arguments per data-model.md "Evaluation rules"
- [X] T058 [US2] Enforce parser limits and positions in `crates/proto123-formula/src/parser.rs` per `data-model.md` `ParseError.pos` and `ParseError.kind` (iterative depth counter, never recursion proportional to input beyond the 256 cap)
- [X] T059 [US2] Implement cycle marking in `crates/proto123-calc/src/lib.rs`: nodes left with in-degree > 0 after Kahn get `Error(Circ)`; recovery path when a cycle is broken (research R9 step 3, FR-014)
- [X] T060 [US2] Implement syntax diagnostics in `crates/proto123-engine/src/edit.rs` and `crates/proto123-engine/src/entry.rs`: commit rejected, caret moved to `ParseError.pos`, status `diag.syntax` / `diag.useQuoteForText`; add the keys to `crates/proto123-engine/src/strings.rs`
- [X] T061 [US2] Render error tokens in the grid and the status-bar diagnostic with the error theme token in `crates/proto123-tui/src/render.rs` (error text is its own non-color cue, FR-035)
- [X] T062 [US2] Create the hostile-input sweep `crates/proto123-engine/tests/panic_hunt.rs` (`#[ignore]`d, run with `--ignored`): extremely long formulas, deep nesting, random Unicode/control characters, refs at grid limits, 10,000-cycles, random key sequences through `InputMapper` + `dispatch`; assert no panic and bounded steps (SC-005, AGENTS rule 30)

**Checkpoint**: US1 + US2 = MVP. All US2 formulas and Edge Cases produce exact results on both targets.

---

## Phase 5: User Story 3 — Select, erase, copy, and move ranges, with Undo and Redo (Priority: P2)

**Goal**: Range selection (Shift+Arrow and `F8`), Range Erase/Copy (tile)/Move with prompts, and ≥ 100-step Undo/Redo.

**Independent Test**: From a scripted grid, run each range command, then Undo/Redo sequences; compare full grid snapshots (raw + display) after each step (spec US3).

### Tests for User Story 3

- [ ] T063 [P] [US3] Write shift/remap tests in `crates/proto123-formula/tests/it/shift.rs`: `Expr::shifted` keeps `$` parts and turns off-grid refs into `#REF!`; `Expr::remap` with `Keep`/`To`/`Invalid` per `data-model.md` `Expr` (shifted, remap)
- [ ] T064 [P] [US3] Write op tests in `crates/proto123-ops/tests/it/ops.rs`: Erase; Copy `B1=A1*2` → `B2:B5` gives `=A2*2`..`=A5*2`; 1×3 into `B1:B10` fills `B1:B9`; single cell into `B1:B10` fills ten; smaller destination pastes once at top-left; `=SUM(A1:A2)` copied right becomes `=SUM(B1:B2)`; Move `A1:A2`→`C1` keeps moved formulas' targets, redirects outside references, overwritten-destination refs become `#REF!`, range refs follow only when fully inside the source; overlapping Copy and Move equal "capture source first" (FR-019–FR-021, research R10)
- [ ] T065 [P] [US3] Write undo tests in `crates/proto123-ops/tests/it/undo.rs`: `CellDelta::invert` round-trip; 100 random mutating commands undone restore the starting grid exactly (raw + display) and redone restore the final grid (SC-006); new command after undo clears redo; capacity drops the oldest step
- [ ] T066 [P] [US3] Write session tests in `crates/proto123-engine/tests/it/range.rs`: `Shift+Down` ×2 selects `A1:A3` and the formula bar shows `A1:A3`; `F8` sets `[EXT]` and plain arrows extend until `F8`/`Esc`; `Delete` erases selection; Copy/Move prompts pre-filled with the selection, accept typed refs and arrow pointing, `Esc` at any prompt leaves no change; `Ctrl+Z`/`Ctrl+Y`; empty-history undo sets `status.nothingToUndo`; navigation/selection create no undo step (US3 sc.1–10, FR-018)
- [ ] T067 [P] [US3] Add US3 scenarios (keyboard-only select/copy/move/undo/redo) to `crates/proto123-testkit/src/scenarios/us3.rs` and register them

### Implementation for User Story 3

- [ ] T068 [US3] Implement `Expr::shifted(dc, dr)` and `Expr::remap(&dyn Fn(RefAddr) -> RefRemap)` in `crates/proto123-formula/src/rewrite.rs`, re-printing the canonical text
- [ ] T069 [US3] Implement `CellDelta` per `data-model.md` `CellDelta.changes` with `invert()` in `crates/proto123-ops/src/delta.rs`, and `UndoStack` per `data-model.md` `UndoStack.undo`, `UndoStack.redo`, `UndoStack` capacity in `crates/proto123-ops/src/undo.rs`
- [ ] T070 [US3] Implement `Op::{SetCell, Erase, Copy, Move}::plan(&CalcSheet) -> CellDelta` in `crates/proto123-ops/src/ops.rs` per research R10 (tiling counts, overlap-safe capture, move remap using the reverse graph from `CalcSheet`)
- [ ] T071 [US3] Route every cell commit through `Op::SetCell` + `UndoStack::push` in `crates/proto123-engine/src/edit.rs`, and implement `range.erase`, `edit.undo`, `edit.redo` in `crates/proto123-engine/src/history.rs` (one `CalcSheet::apply` per step); register ids
- [ ] T072 [US3] Implement selection commands `nav.extendSelection` and `nav.toggleExtendMode` in `crates/proto123-engine/src/nav.rs` per `data-model.md` `Session.anchor` and `Session.extend_mode`
- [ ] T073 [US3] Implement `Prompt::{RangeSource, RangeDest}` per `data-model.md` `Session.prompt`, `DIALOG` mode, and `prompt.input`/`prompt.caret`/`prompt.submit`/`prompt.cancel` plus arrow pointing in prompts in `crates/proto123-engine/src/prompt.rs`; `range.copy` and `range.move` run the op as one undo step on submit; invalid range text gives a status diagnostic, never a partial apply
- [ ] T074 [US3] Render selection (inverted video + range text in the formula bar of the control line), `[EXT]` in the status bar, and the prompt line with its text field and `[OK]`/`[Cancel]` controls recorded as `PromptField`/`PromptButton` in `crates/proto123-tui/src/render.rs`
- [ ] T075 [US3] Map `Shift+Arrow`, `F8`, `Delete`, `Ctrl+Z`, `Ctrl+Y`, and prompt keys in `crates/proto123-tui/src/input.rs`; add status/prompt keys to `crates/proto123-engine/src/strings.rs`

**Checkpoint**: US3 Independent Test passes on both targets (Copy/Move are reachable from the keyboard via the prompts; their menu entries arrive in US5).

---

## Phase 6: User Story 5 — Navigate the Slash Command Bar by keyboard and switch themes (Priority: P2)

> Ordered before US4 because the mouse story clicks menu items that this story creates. Both are P2.

**Goal**: Two-line `/` menu with the exact tree, letter triggers, wrap-around, `Esc` up a level, File New confirmation, theme cycling.

**Independent Test**: Drive key sequences; assert menu level, highlight, second-line text, executed command, final mode; run `menu::verify` on the catalog and seeded bad catalogs (spec US5).

### Tests for User Story 5

- [ ] T076 [P] [US5] Write `menu::verify` tests in `crates/proto123-engine/tests/it/menu.rs`: the shipped catalog passes and equals the exact tree in contracts/commands.md; seeded catalogs with an unregistered leaf id, duplicate trigger, trigger not equal to the label's first letter, and a missing string key each report the menu path (SC-011)
- [ ] T077 [P] [US5] Write menu-session tests in `crates/proto123-engine/tests/it/menu_flow.rs`: US5 sc.1–7 (`/` opens with first item highlighted and described, `Left`/`Right` wrap, `Enter` and letters activate, `Esc` goes up then to READY, `/ R E` erases the selection, `/ R C` opens the Copy prompt, File New prompt only when non-empty with default No, confirmed New clears workbook and undo history and Undo then reports nothing to undo, `/ V T` cycles all themes back to the first and each cycle returns `Redraw::Full` with no undo step)
- [ ] T078 [P] [US5] Write theme tests in `crates/proto123-theme/tests/it/contrast.rs`: WCAG contrast for every built-in theme (≥ 4.5 text tokens, ≥ 3.0 `border`/`active_cell`/`menu_highlight`), ANSI-16 tokens measured against the stored VGA console palette, `console16` uses only `Ansi16` with `allow_bold = false`, `mono` uses only black/white + ASCII glyphs, and every built-in theme carries the same `GridMetrics` (16 px `monospace`, 10×19 cell, 5 px origin, research R2) (SC-008, FR-034, FR-048)
- [ ] T079 [P] [US5] Add US5 scenarios to `crates/proto123-testkit/src/scenarios/us5.rs` and register them

### Implementation for User Story 5

- [ ] T080 [P] [US5] Implement WCAG luminance/contrast and the VGA palette constant in `crates/proto123-theme/src/contrast.rs`, and the `console16` and `mono` built-in themes in `crates/proto123-theme/src/builtin.rs` per `data-model.md` `Theme.allow_bold` and `ThemeId` (research R6)
- [ ] T081 [US5] Implement `MenuNode` per `data-model.md` `MenuNode.trigger` and the static `CATALOG` in `crates/proto123-engine/src/menu_catalog.rs` with the exact tree; add menu labels and descriptions to `crates/proto123-engine/src/strings.rs`; implement `menu::verify(catalog, registry) -> Vec<MenuViolation>` in `crates/proto123-engine/src/menu.rs` (research R11)
- [ ] T082 [US5] Implement `MenuState` per `data-model.md` `Session.menu` and the commands `menu.open`, `menu.openAt`, `menu.move`, `menu.hover`, `menu.activate`, `menu.trigger`, `menu.back`, `menu.close` in `crates/proto123-engine/src/menu.rs`; a leaf needing input (Copy, Move, New with content) enters `DIALOG` via the US3 prompts
- [ ] T083 [US5] Implement `file.newWorkbook` with `Prompt::ConfirmNew { focus }` (default No) in `crates/proto123-engine/src/prompt.rs` and `crates/proto123-engine/src/history.rs` (reset `CalcSheet`, `UndoStack::clear`) per FR-022; implement `view.cycleTheme` and `session.setTheme` in `crates/proto123-engine/src/session.rs` (theme change returns `Redraw::Full`, no undo step)
- [ ] T084 [US5] Render the two-line Slash Command Bar in `crates/proto123-tui/src/render.rs`: root bar always visible in READY, current level items with `>` + inverted highlight, second line description or submenu preview, `[Yes]`/`[No]` confirm prompt; record `MenuItem { level, index }` and `PromptButton` targets in the `LayoutMap`
- [ ] T085 [US5] Map `/`, menu `Left`/`Right`/`Enter`/`Esc`, letter triggers, and confirm-prompt keys in `crates/proto123-tui/src/input.rs`
- [ ] T086 [US5] Implement `cargo xtask menu-commands` in `xtask/src/menu.rs` calling `proto123_engine::menu::verify(CATALOG, REGISTRY)` and exiting 0/1 per contracts/xtask.md (xtask depends on `proto123-engine`; register the edge in `xtask/src/layers.rs` as `Tooling`)

**Checkpoint**: every command in this slice is reachable by keyboard, including via the menu; `cargo xtask menu-commands` passes.

---

## Phase 7: User Story 4 — Operate everything with the mouse (Priority: P2)

**Goal**: Click, Shift+click, double-click edit, drag-select with auto-scroll, wheel, formula-bar caret click, pointing with the mouse, menu hover/click, click-outside close — interpreted only in `proto123-tui`.

**Independent Test**: Feed timestamped raw pointer events through `InputMapper`; assert resulting commands and session state without a browser (spec US4).

### Tests for User Story 4

- [ ] T087 [P] [US4] Write `pixel_to_cell` tests in `crates/proto123-tui/tests/it/pixel.rs`: DPR 1, 1.25, 2; cell edges; origin offset; negative and out-of-canvas points → `None` per `data-model.md` `CellMetrics.dpr` and `CellMetrics.origin_x`/`origin_y` (FR-031 step 1)
- [ ] T088 [P] [US4] Write gesture tests in `crates/proto123-tui/tests/it/gestures.rs`: click selects `C5`, `Shift+click` `E9` → `C5:E9`; double-click within 500 ms on the same cell → EDIT with caret at end, 501 ms or a different cell → two clicks (per `data-model.md` `InputMapper.last_press`); drag `B2`→`D6`; drag past the bottom edge auto-scrolls and extends; wheel notch = 3 rows, `Shift+wheel` and `dx` scroll columns; formula-bar click at char *n* → EDIT caret *n* clamped; `=SUM(` + click `B2` / drag `B2:B9` → POINT with live-updating reference; click elsewhere while not pointing commits (or shows the syntax diagnostic) and selects; menu hover highlight + second line; click `Edit` then `Undo`; click outside closes menu with no change; prompt grid click/drag fills the range text; below 80×24 pointer events produce no mutating command (US4 sc.1–9)
- [ ] T089 [P] [US4] Add US4 pointer scenarios to `crates/proto123-testkit/src/scenarios/us4.rs` (cell-position events; the web suite converts them to pixels through its `CellMetrics`) and register them

### Implementation for User Story 4

- [ ] T090 [P] [US4] Implement `pixel_to_cell` in `crates/proto123-tui/src/pixel.rs` over the Phase 2 `CellMetrics` (T023) per `data-model.md` `CellMetrics.dpr`, `CellMetrics.origin_x`, `CellMetrics.origin_y`
- [ ] T091 [US4] Implement `point.setReference` and POINT transitions in `crates/proto123-engine/src/point.rs` per `data-model.md` `Session.edit` (`EditState.point`: `PointState.insert_at`, `PointState.len`, `PointState.anchor`, `PointState.cursor`): eligible only when the buffer is a formula and the caret follows an operator, `(`, `,`, `;`, `:` (FR-029); arrow keys and `F8` in POINT; typing returns to VALUE/EDIT keeping the reference; register the id
- [ ] T092 [US4] Implement `InputMapper` pointer handling in `crates/proto123-tui/src/input.rs` per `data-model.md` `InputMapper.last_press`, `InputMapper.drag`, `InputMapper.hover`: press/move/up/leave, double-click rule, drag start/extend/end with `GridEdge` auto-scroll, wheel, formula-bar/address-box/menu/prompt targets, Shift modifier, click-outside menu close; all decisions from the current `LayoutMap` (FR-028–FR-031)
- [ ] T093 [US4] Record `FormulaBar { char_index }`, `GridEdge(..)`, hovered `MenuItem`, and `PromptField { char_index }` targets precisely in `crates/proto123-tui/src/render.rs`, and draw the hover cue (`>` + inverted) for menu items and prompt buttons
- [ ] T094 [P] [US4] Extend `apps/proto123-term/src/translate.rs` with crossterm mouse rows (`Down`/`Up` → `PointerDown/Up` `Pos::Cell`, `Drag`/`Moved` → `PointerMove`, `Scroll*` → `Wheel ±1`) per contracts/raw-input.md
- [ ] T095 [P] [US4] Extend `apps/proto123-wasm/src/translate.rs` with `pointerdown/move/up/leave` (`Pos::Pixel` → `pixel_to_cell`) and `wheel` (sign → ±1 notch) rows, and add `on_pointer`/`on_wheel` exports in `apps/proto123-wasm/src/bridge.rs` per contracts/wasm-bridge.md
- [ ] T096 [US4] Forward `pointerdown`/`pointermove`/`pointerup`/`pointerleave` (with `setPointerCapture`) and `wheel` (`preventDefault`) primitives in `web/host.js`, no interpretation (FR-037)

**Checkpoint**: US4 scenarios pass in the core and through both translations; the full slice is mouse-operable in the browser.

---

## Phase 8: User Story 8 — Use Proto123 in a native terminal, incl. the Linux console (Priority: P2)

**Goal**: Profile detection and overrides, mouse capture, console profile rendering, full restoration on every exit path, resize handling.

**Independent Test**: Shared scenario suite on the native translation; console-profile frames into `TestBackend` contain only ANSI-16 colors and allowed glyphs; guard/panic restore test; manual Linux-console smoke (spec US8).

### Tests for User Story 8

- [ ] T097 [P] [US8] Write `CapabilityProfile::detect` table tests in `crates/proto123-theme/tests/it/profile.rs`: `TERM=linux`→`Console`, `COLORTERM=truecolor|24bit`→`Full`, `*256color`→`Full`, `xterm`/`screen`/`tmux`/`vt*`→`Ansi16`, unknown/empty→`Conservative` with ASCII glyphs (research R5)
- [ ] T098 [P] [US8] Write console-profile frame tests in `crates/proto123-tui/tests/it/console.rs`: render US1–US5 states with the `Console` profile into `TestBackend`; assert every cell's fg/bg is a named ANSI-16 color, no `BOLD`/`UNDERLINED` modifier, glyphs are within the single-line box set (or ASCII set when forced), and each state cue (active cell, selection, menu highlight, `[EXT]`, errors) is present as glyph/bracket/inverse (FR-035, FR-048, US8 sc.3, sc.7); repeat the cue assertions for every built-in theme (`high-contrast`, `console16`, and `mono` with ASCII glyphs) so no state depends on color alone (SC-008)
- [ ] T099 [P] [US8] Write lifecycle tests in `apps/proto123-term/tests/it/lifecycle.rs`: `guard_restores_on_panic` drives `TerminalGuard` + panic hook against a fake writer and asserts the restore sequence (disable mouse capture, leave alternate screen, disable raw mode, show cursor) on normal drop, on `Err`, and on panic; argument parsing (`--theme`, `--glyphs`, `--help`, `--version`, positional arg and unknown values → exit 2) per contracts/native-cli.md (SC-014)

### Implementation for User Story 8

- [ ] T100 [P] [US8] Implement `CapabilityProfile` per `data-model.md` `CapabilityProfile` (`detect`, `default_theme`, `default_glyphs`) in `crates/proto123-theme/src/profile.rs`
- [ ] T101 [US8] Implement hand-written argument parsing in `apps/proto123-term/src/args.rs` and startup in `apps/proto123-term/src/main.rs`: read `TERM`/`COLORTERM`, `detect`, apply `--theme`/`--glyphs` overrides via `session.setTheme`, exit codes 0/1/2 (contracts/native-cli.md)
- [ ] T102 [US8] Extend `apps/proto123-term/src/guard.rs`: `EnableMouseCapture` on start / `DisableMouseCapture` on restore, panic hook that restores before printing, internal errors printed to stderr only after restoration (FR-046, research R4)
- [ ] T103 [US8] Handle `Event::Resize` as a full redraw and the below-80×24 frame in `apps/proto123-term/src/main.rs` (no extra logic beyond forwarding `Resize`, US8 sc.6)
- [ ] T104 [US8] Confirm the native scenario suite in `apps/proto123-term/tests/it/scenarios.rs` runs all US1–US5 scenarios with the `Console` profile as well as `Full`, and that the no-mouse path completes every scenario (US8 sc.5, SC-012)

**Checkpoint**: `cargo test -p proto123-term` green; manual smoke in a desktop terminal and in a Linux virtual console per quickstart.md §5.

---

## Phase 9: User Story 6 — Screen-reader and low-vision access in the browser (Priority: P3)

**Goal**: One live region with concise announcements; contrast and non-color cues verified.

**Independent Test**: Check text sent to the live region after scripted navigation, mode, menu and error changes; contrast per theme (spec US6).

### Tests for User Story 6

- [ ] T105 [P] [US6] Write announcement tests in `crates/proto123-engine/tests/it/announce.rs`: moving to `C4` gives `C4, formula =SUM(C1:C3), value 150, READY`; mode change, menu highlight (`Menu Range, Copy: copy a range to another place, MENU`), and new status/error each produce exactly one announcement; no announcement when nothing relevant changed; numbers use `format_full` (`=0.1+0.2` → `value 0.3`) (FR-036, US2 sc.9)
- [ ] T106 [P] [US6] Write a page-shape test in `apps/proto123-wasm/tests/it/page.rs` that parses `web/index.html` and asserts exactly one `<canvas>` and one `[aria-live]` element and that `web/host.js` creates no elements (no `createElement`, `appendChild`, `innerHTML`) (US6 sc.1, SC-009)

### Implementation for User Story 6

- [ ] T107 [US6] Build announcements in `crates/proto123-engine/src/announce.rs` from string-catalog templates per `data-model.md` `Session.announce`, emitted only on cell move, mode change, menu highlight change, or new status (contracts/wasm-bridge.md "Announcement text"); add keys to `crates/proto123-engine/src/strings.rs`
- [ ] T108 [US6] Return the drained announcement from every export in `apps/proto123-wasm/src/bridge.rs`, and write it to `#live.textContent` in `web/host.js` (the only DOM write)

**Checkpoint**: live-region text matches the contract; contrast tests (T078) green for every theme.

---

## Phase 10: User Story 7 — Maintainers run, ship, and stay within the architecture (Priority: P3)

**Goal**: One-command local serve, release size report, container image with `application/wasm`, CI running every guardrail. (The layer check landed in Phase 2 and the menu-command check in US5.)

**Independent Test**: Layer and menu checks pass and catch seeded violations; build the image, GET the `.wasm`, check the content type; size report printed (spec US7).

### Tests for User Story 7

- [ ] T109 [P] [US7] Write a unit test in `xtask/tests/it/http.rs` for the response-parsing helper used by `docker-check` (status line and `Content-Type` header extraction from a canned HTTP/1.1 response), and one for the size-report line format in `xtask/tests/it/wasm.rs`

### Implementation for User Story 7

- [ ] T110 [P] [US7] Implement `cargo xtask wasm` in `xtask/src/wasm.rs`: `cargo build --target wasm32-unknown-unknown` for each L0–L5 crate, `trunk build --release`, report `wasm: <path> raw=<bytes> gzip=<bytes>` (gzip level 9 via `flate2`); missing `trunk` → exit 2 (contracts/xtask.md, FR-042)
- [ ] T111 [P] [US7] Create `Dockerfile` (stage 1 `rust:1.99-slim` + wasm32 target + checksum-verified pinned Trunk 0.22.2 → `trunk build --release`; stage 2 `nginx:alpine` copying `dist/`) and `docker/nginx.conf` (`types { application/wasm wasm; }`, gzip incl. wasm) per research R13
- [ ] T112 [US7] Implement `cargo xtask docker-check` in `xtask/src/docker.rs`: `docker build -t proto123:check .`, run on a free port, `std::net::TcpStream` GET `/` and the `.wasm` path, assert 200 and `Content-Type: application/wasm`, always stop the container; missing `docker` → exit 2
- [ ] T113 [US7] Implement `cargo xtask ci` in `xtask/src/main.rs` running `fmt --check`, `clippy --workspace --all-targets -- -D warnings`, `layers`, `menu-commands`, `test --workspace`, `wasm` in order and naming the first failing step
- [ ] T114 [US7] Create `.github/workflows/ci.yml`: fmt, clippy, test, `xtask layers`, `xtask menu-commands`, `xtask wasm` (uploads the size line as a job summary), `cargo build -p proto123-term` on `ubuntu-latest`, `macos-latest`, `windows-latest`, and `xtask docker-check` on Linux (in scope per plan.md Constitution Check; AGENTS rule 18)
- [ ] T115 [US7] Update `docs/development.md` sections "Native Terminal App", "Local Web App", "Docker Execution", and "Workspace Gate & `xtask` Commands" with the exact commands from quickstart.md (marking `proto123-cli`, `ods-conformance`, and `?file=` loading as Spec 002)

**Checkpoint**: `cargo xtask ci` green locally; `trunk serve` works offline after load; `cargo xtask docker-check` passes.

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Docs reconciliation required by the spec, performance evidence, final sweeps.

- [ ] T116 [P] Update `docs/interaction-spec.md` (§2 was done in T018a) §3 (FR-023 bindings, `F8`, `Delete` only — no `Backspace` erase, no `Ctrl+C`/`Ctrl+X`/`Ctrl+V`, `Ctrl+Q` native quit), and §5 (exact menu tree with Copy/Move under `Range`) per spec Assumptions "Menu placement"
- [ ] T117 [P] Update `docs/feature-names.md` §4 so top-level labels and leaves match the exact tree (`File`, `Edit`, `Range`, `View`)
- [ ] T118 [P] Update `docs/parity-checklist.md` with every `CommandId` in contracts/commands.md and its Keyboard / Mouse / Menu path (CLI column marked "Spec 002"), satisfying SC-007
- [ ] T119 [P] Update `docs/architecture.md` "Current state" with the crates that now exist, the Ratzilla drawing-only decision (research R2–R3), and the allowed same-layer edges
- [ ] T120 [P] Create `crates/proto123-tui/examples/frame_budget.rs`: build the 10,000-cell / 5,000-formula reference sheet and report p50/p99 input→buffer-ready time over arrow, click, drag, and wheel events against `TestBackend` (SC-002; reported, never asserted in CI)
- [ ] T121 Extend `crates/proto123-engine/tests/panic_hunt.rs` with random pointer sequences, menu/prompt sequences, and resize-below-minimum sequences covering every `CommandId` (SC-005)
- [ ] T122 Run the full gate and record results: `cargo xtask ci`, `cargo test -p proto123-engine --test panic_hunt -- --ignored`, `cargo build -p proto123-term`, `cargo xtask docker-check`, `cargo run --release -p proto123-tui --example frame_budget`; then run quickstart.md §5–§6 manual checks (browser keyboard/mouse/zoom/offline, Linux virtual console smoke) and list them as manual in the PR (AGENTS rule 34); record the `frame_budget` p50/p99 per event kind and the browser DevTools Performance input→frame result in the PR as the constitution Principle I / SC-002 evidence — any p99 above 16 ms means the slice is not complete

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: none.
- **Foundational (Phase 2)**: after Setup; blocks every story.
- **US1 (Phase 3)**: after Phase 2. Provides the formula parser/evaluator and `CalcSheet` that every later story uses.
- **US2 (Phase 4)**: after US1 (extends the same evaluator, parser, and recalc).
- **US3 (Phase 5)**: after US1; independent of US2 (US2 only hardens semantics).
- **US5 (Phase 6)**: after US3 (menu leaves `Range Erase/Copy/Move`, `Edit Undo/Redo` call US3 commands).
- **US4 (Phase 7)**: after US5 (clicks menu items and prompt controls) and US1 (pointing into the edit buffer).
- **US8 (Phase 8)**: after Phase 2 for detection/guard work (T097, T099–T103 can start right after Phase 2); T098 and T104 need US1–US5 states.
- **US6 (Phase 9)**: after US5 (menu announcements); T106 only needs Phase 2.
- **US7 (Phase 10)**: after Phase 2 (T110–T115 need only the shells); `xtask ci` includes `menu-commands`, so T113 needs T086.
- **Polish (Phase 11)**: after all stories.

### Story completion order

```
Setup → Foundational ─┬─ US1 ─┬─ US2
                      │       └─ US3 ── US5 ─┬─ US4
                      │                      └─ US6
                      ├─ US8 (detection/guard early; console frames after US5)
                      └─ US7 (after T086 for xtask ci)
                                            → Polish
```

### Within each story

Tests first (they must fail before implementation), then lower layers before higher (formula → calc → ops → engine → tui → apps), then the shared scenario registration so both targets are checked.

---

## Parallel Execution Examples

### Phase 2

```
T008 (xtask layer tests) ‖ T010 (coord tests) ‖ T013 (value tests) ‖ T015 (theme types) ‖ T018 (strings) ‖ T018a (interaction-spec §2)
then T011 ‖ T014 ‖ T023 ‖ T024;   later T030 ‖ T031 ‖ T034
```

### User Story 1

```
T036 ‖ T037 ‖ T038 ‖ T039 ‖ T040 ‖ T040a ‖ T040b   (tests, different files)
T041 (lexer) runs while T045 (graph) is written; T047 (entry) in parallel with T042–T044
```

### User Story 2

```
T052 ‖ T053 ‖ T054 ‖ T055 ‖ T056;  T057 (eval) ‖ T059 (calc cycles) ‖ T061 (render)
```

### User Story 3

```
T063 ‖ T064 ‖ T065 ‖ T066 ‖ T067;  T068 (formula rewrite) ‖ T069 (delta/undo) then T070
```

### User Story 5

```
T076 ‖ T077 ‖ T078 ‖ T079;  T080 (themes) ‖ T081 (catalog)
```

### User Story 4

```
T087 ‖ T088 ‖ T089;  T090 ‖ T094 ‖ T095 after T092
```

### User Story 8

```
T097 ‖ T099 ‖ T100 right after Phase 2;  T098 after US5
```

### User Story 7

```
T109 ‖ T110 ‖ T111
```

---

## Implementation Strategy

### MVP first (US1 + US2, both P1)

1. Phase 1 Setup → Phase 2 Foundational (both shells already draw and navigate).
2. Phase 3 US1 → validate the US1 Independent Test on both translations, run the native app and `trunk serve`.
3. Phase 4 US2 → validate exact results, cycles, diagnostics, and `panic_hunt`.
4. **Stop and validate**: a correct, keyboard-driven calculating grid on web and native. This is the MVP.

### Incremental delivery

5. US3 (ranges + undo) → US5 (menu + themes) → US4 (mouse) → US8 (terminal profiles/lifecycle) → US6 (live region) → US7 (size report, container, CI) → Polish.
6. Each story ends with its scenarios added to `proto123-testkit`, so both targets are re-checked at every step (FR-049).

### Notes

- Never add a `CommandId` without its handler and its key/mouse path in the same task (research R11; no stub arms).
- Never silence lints with `#[allow]`; no `unwrap`/`expect`/`panic`/indexing in non-test code (AGENTS rule 13).
- Every new user-facing string goes into `crates/proto123-engine/src/strings.rs` (FR-027).
- `ATTRIBUTION.md` stays unchanged: no fonts or fixtures are added in this slice.
- Commit after each task or logical group; ff-merge short branches into local `main`.
