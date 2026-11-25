# Research: Foundational MVP (Spec 001)

Phase 0 output. Every "NEEDS CLARIFICATION" from the Technical Context is resolved here. Versions were checked against crates.io on 2026-11-25; Ratzilla 0.3.1 source (MIT/Apache-2.0, a permissive non-spreadsheet crate) was read to confirm its API.

---

## R1 — Toolchain and edition

- **Decision**: Rust 1.99.1, edition 2024, pinned in `rust-toolchain.toml` with `targets = ["wasm32-unknown-unknown"]` and `components = ["clippy", "rustfmt"]`. `rust-version = "1.99"` in the workspace.
- **Rationale**: Constitution says "latest stable"; the installed toolchain is 1.99.1. Pinning makes CI and Docker reproducible, and rustup installs the wasm target on first use.
- **Alternatives**: Floating `stable` — rejected, non-reproducible builds.

## R2 — Web renderer (constitution requires this decision in the first rendering plan)

- **Decision**: Ratzilla 0.3 **`CanvasBackend`** draws the shared Ratatui buffer on one `<canvas>`. Ratzilla's DOM backend is never used; its WebGL2 backend is not used in this slice.
- **Rationale**: Ratzilla's `CanvasBackend::flush` compares the new buffer to the previous one and redraws only changed cells (`resolve_changed_cells`), batching background and glyph passes to cut WASM↔JS calls — exactly FR-033. Combined with Ratatui's own `Terminal` diff, a cursor move touches a handful of cells.
- **Known properties to design around**: fixed cell size 10×19 CSS px with a 5 px origin translate and `16px monospace` font; exposes `cell_size_css_px()`; scales by `devicePixelRatio` internally.
- **Alternatives**: Custom Canvas 2D painter — allowed only if Ratzilla misses 16 ms; no evidence it will. WebGL2 (beamterm) — faster but heavier and needs a font atlas; revisit only if Canvas misses the budget.

## R3 — Input path on the web (bypass Ratzilla events)

- **Decision**: `web/host.js` attaches `keydown`, `pointerdown/move/up`, `wheel`, `ResizeObserver`, and DPR-change listeners and forwards **primitive values plus `event.timeStamp`** to `#[wasm_bindgen]` exports. Ratzilla's `on_key_event`/`on_mouse_event` are not used.
- **Rationale**: Ratzilla's `MouseEvent` is already converted to cell coordinates (violates "pixel→cell in `proto123-tui`, unit-tested", FR-031), carries no timestamp (FR-030 needs raw timestamped events), and has no wheel kind (FR-028). Forwarding raw values keeps the host thin and the interpretation in Rust.
- **Pixel mapping inputs**: host sends `offsetX/offsetY` in CSS px and `devicePixelRatio`; `proto123-tui::CellMetrics { cell_w, cell_h, origin_x, origin_y, dpr }` is built by the wasm app from Ratzilla's `cell_size_css_px()` and its 5 px origin. `pixel_to_cell` is pure and unit-tested for DPR 1/1.25/2, edges, negative and out-of-canvas points.
- **Alternatives**: Use Ratzilla events and add timestamps in Rust via `performance.now()` — rejected: still no wheel, and pixel mapping would stay outside our tested function.

## R4 — Native terminal stack

- **Decision**: `crossterm` 0.29 + `ratatui` 0.30 `CrosstermBackend`. Startup: raw mode, alternate screen, `EnableMouseCapture` (SGR: press, release, drag, motion, wheel), hidden cursor. A `TerminalGuard` (`Drop`) plus a panic hook restore everything (disable mouse capture, leave alternate screen, disable raw mode, show cursor); `main` returns `ExitCode` after the guard drops and prints any internal error to stderr only after restoration.
- **Timestamps**: crossterm events carry none; the app stamps each with milliseconds since start from `std::time::Instant` (monotonic) before handing it to the core.
- **Rationale**: Pure Rust, no X11/GPU, supports Linux console, macOS, Windows ConPTY. Release profile keeps `panic = "unwind"` natively so `Drop` runs (SC-014).
- **Alternatives**: termion (no Windows), termwiz (heavier).

## R5 — Capability profile detection and override

- **Decision**: The rule lives in the core: `proto123_theme::CapabilityProfile::detect(term: Option<&str>, colorterm: Option<&str>) -> Profile`. `apps/proto123-term` only reads the two env vars and passes them in. Rules: `TERM=linux` → `Console` (theme `console16`, single-line box glyphs); `COLORTERM` ∈ {`truecolor`,`24bit`} → `Full` (theme `high-contrast`, box glyphs); `TERM` containing `256color` → `Full`; other known `xterm*`/`screen*`/`tmux*`/`vt*` → `Ansi16` (`console16`, box glyphs); unknown/empty → `Conservative` (`console16`, ASCII glyphs). Override flags `--theme <id>` and `--glyphs <box|ascii>` win over detection ([contracts/native-cli.md](contracts/native-cli.md)). The web always uses `Full`.
- **Rationale**: Keeps the native front end rule-free (FR-045) and makes detection unit-testable.

## R6 — Themes, colors, contrast

- **Decision**: L0 `proto123-theme` owns `Color { Rgb(u8,u8,u8), Ansi16(Ansi16) }`, `GlyphSet { Box, Ascii }`, `Theme { id, tokens: ThemeTokens }` with named tokens (`grid_text`, `grid_bg`, `header_text`, `header_bg`, `active_cell`, `selection`, `menu_text`, `menu_highlight`, `border`, `error_text`, `status_text`, `badge`…). Three built-ins: `high-contrast` (RGB), `console16` (ANSI-16 only, no bold/underline modifiers), `mono` (black/white only + ASCII glyphs). `proto123-tui` maps tokens to `ratatui::style::Color` (ANSI-16 → named ratatui colors so terminals use their own palette).
- **Contrast check**: WCAG relative-luminance math in `proto123-theme`; ANSI-16 tokens are evaluated against the Linux console's default VGA palette table (stored as a constant) so FR-048 is testable. Test asserts ≥ 4.5:1 text and ≥ 3:1 controls for every theme (SC-008).
- **Non-color cues** (FR-035), using only glyphs every console font has: active cell = inverted video **plus** bracketed column header and row number (`[B]`, `[  3]`) and `[B3]` in the control line; range selection = inverted video plus the range text (`A1:A3`) in the control line, so in `mono` the active cell is still told apart by its bracketed headers; menu highlight/hover = inverted video plus `>` before the label; extend mode = `[EXT]` in the status bar; errors = their token text.

## R7 — Number model and display (FR-004, FR-006)

- **Decision**: `f64` (IEEE-754) everywhere (clarified). Display is a pure function `proto123_value::format_general(n: f64, width: usize) -> String` implementing the four FR-006 steps, using `format!("{:.*e}", 14, n)` to get 15 significant digits, then trimming/rounding via decimal string manipulation (half away from zero on the decimal digit string, not on binary). `-0` → `0`. Non-finite values never reach display: arithmetic maps them to `#VALUE!` (or `#DIV/0!`).
- **Rationale**: No dependency; string-based rounding avoids double-rounding surprises (`=0.1+0.2` → `0.3`, `=1-0.9` → `0.1`).
- **Alternatives**: `ryu` + custom rounding — extra dependency for no gain.

## R8 — Formula language

- **Decision**: Hand-written lexer + Pratt parser in `proto123-formula`. Binding powers: `+ -` (10, left), `* /` (20, left), `^` (30, left), prefix `- +` (40) — so `-2^2 = 4` and `2^3^2 = 64` (FR-008). Function names case-insensitive, args separated by `,` or `;`. `@SUM(` at the start of an entry is accepted as shorthand and printed canonically as `=SUM(`. Leading `+` is canonicalised to `=`. Error literals (`#REF!`) are valid primary expressions so rewritten references round-trip as text. Limits: input ≤ 8,192 chars and nesting depth ≤ 256 — exceeding either is a syntax diagnostic, never a stack overflow.
- **Unknown functions** parse successfully into `Expr::Call { name, .. }` and evaluate to `#NAME?`, so the text is kept for `F2` editing (US2 sc.2).
- **Grammar** in [contracts/formula-grammar.md](contracts/formula-grammar.md).

## R9 — Dependency graph and recalculation (FR-013, FR-014)

- **Decision**: `proto123-calc::DependencyGraph` stores, per formula cell, its precedents split into `cells: Vec<CellAddr>` and `ranges: Vec<CellRange>`; reverse edges as `HashMap<CellAddr, HashSet<CellAddr>>` for single cells plus a `Vec<(CellRange, CellAddr)>` range index scanned linearly. On a change to cells `S`:
  1. Update precedents for edited formula cells only.
  2. Iterative BFS from `S` collecting the dirty set **and recording each discovered edge** `x → dependent`.
  3. Kahn's algorithm over the recorded edges; nodes never reaching in-degree 0 are cycle members or downstream of a cycle → `Error(Circ)`.
  4. Evaluate in topological order reading cached precedent values; return `RecalcReport { evaluated: Vec<CellAddr> }` (its length is what SC-003 asserts).
- **Range aggregates** iterate `min(range area, populated cells)` so `=SUM(A1:XFD1048576)` over 10 populated cells costs ~10 lookups.
- **Rationale**: All loops are iterative (10,000-deep chains, 10,000-long cycles); cost is O(dirty edges); linear range index is ≤ 5,000 entries at the reference size, a few µs.
- **Alternatives**: R-tree/interval tree for ranges — deferred until a profile shows the linear scan matters; full-sheet recalculation — forbidden by Principle VI.

## R10 — Undo, Copy (tile), Move

- **Decision**: Every mutating command produces one `CellDelta { changes: Vec<(CellAddr, Option<CellContent> /*before*/, Option<CellContent> /*after*/)> }`; undo applies `before`, redo applies `after`, each as one batch followed by one recalc. Stack capacity 1,000 steps (≥ 100 required); oldest dropped beyond that. File New clears both stacks.
  - **Copy**: capture the whole source first (overlap-safe, FR-021); tile count per axis = `max(1, dest_len / src_len)` when `dest_len ≥ src_len`, else 1 (FR-019); each tile shifts relative refs by its offset via `Expr::shifted(dc, dr)`; out-of-grid → `#REF!`.
  - **Move**: capture source; compute affected cells = dest ∪ source; formulas elsewhere that reference a moved cell (found via the reverse graph) are rewritten with `Expr::remap`; a reference to a single moved cell follows it; a range reference follows only if the range lies entirely inside the source (otherwise unchanged); references to overwritten destination cells (not in source) become `#REF!`. Moved formulas keep their targets (relative refs adjusted by the inverse offset so they point at the same cells; refs to other moved cells follow).
- **Rationale**: O(changed cells) memory; overlap-safe by construction; one undo step per command.

## R11 — Command registry and menu-command check

- **Decision**: `CommandId` is a Rust `enum`; `REGISTRY: &[CommandSpec { id, name /* "range.copy" */, undoable }]` lists every variant; `dispatch` is an exhaustive `match` (so every registered command has a handler at compile time; no stub arm is permitted — a `NotImplemented` arm would fail review and the check). `menu_catalog.rs` is static data (`MenuNode { label_key, trigger, kind: Branch(&[MenuNode]) | Leaf(CommandId) }`). `proto123_engine::menu::verify(catalog, registry) -> Vec<MenuViolation>` checks: leaf id registered, unique trigger letters per level, trigger equals the label's first letter, labels resolve in the string catalog. `cargo xtask menu-commands` calls it (xtask depends on `proto123-engine`); unit tests feed seeded bad catalogs (SC-011).

## R12 — Layer check

- **Decision**: `xtask/src/layers.rs` holds `const LAYERS: &[(&str, Layer)]` for every planned crate (L0–L6, `Support`, `Tooling`). `cargo xtask layers` reads `cargo metadata` (via `cargo_metadata`), and the pure function `check(packages, LAYERS)` reports: unregistered workspace member; normal/build dependency on a higher layer; `proto123-testkit` used outside `[dev-dependencies]`; any L0–L4 crate depending on `ratatui`, `ratzilla`, `crossterm`, `web-sys`, `wasm-bindgen`; L5 depending on a backend crate; a graphical-session or GPU crate (`x11`, `x11rb`, `x11-clipboard`, `wayland-client`, `wgpu`, `glutin`) anywhere in `proto123-term`'s normal dependency tree (FR-044). Same-layer acyclic edges are allowed. Seeded-violation unit tests cover each rule.

## R13 — Web build, serving, container, size report

- **Decision**: `Trunk.toml` targets `web/index.html` (`<link data-trunk rel="rust" href="../apps/proto123-wasm/Cargo.toml" data-wasm-opt="z" data-cargo-profile="release-wasm" />`; JS/CSS copied with `rel="copy-file"`/`rel="css"`). Local: `trunk serve` (dev) or `trunk build --release` + any static server; the app needs no network after load (no fetches in this slice). Exports are reached through `window.wasmBindings` after Trunk's `TrunkApplicationStarted` event.
  - **Profile** `release-wasm`: `opt-level="z"`, `lto=true`, `codegen-units=1`, `panic="abort"` (the core never panics by rule; `console_error_panic_hook` reports anything unexpected).
  - **Dockerfile**: stage 1 `rust:1.99-slim` + `wasm32` target + pinned Trunk release binary (checksum-verified) → `trunk build --release`; stage 2 `nginx:alpine` with `docker/nginx.conf` (`types { application/wasm wasm; }`, gzip incl. wasm).
  - **`cargo xtask wasm`**: builds every L0–L5 crate for `wasm32-unknown-unknown`, runs `trunk build --release`, then reports raw and gzip (level 9, `flate2`) size of `dist/*.wasm`. No ceiling yet (constitution TODO); the first report is the baseline.
  - **`cargo xtask docker-check`**: `docker build`, `docker run -d -p <free port>`, plain `std::net::TcpStream` HTTP/1.1 GET of `/` and the `.wasm` asset, asserts `200` and `Content-Type: application/wasm`, then stops the container.
- **Alternatives**: `wasm-pack` + hand-written bundling — more host JS; Caddy runtime — nginx config is already in `docs/web-integration.md`.

## R14 — Shared scenario suite and performance evidence

- **Decision**: `proto123-testkit` defines scenarios as data: a start grid plus a list of target-neutral `RawEvent`s, and expected snapshots (`SessionSnapshot`: mode, active cell, selection, buffer/caret, status key, grid raw+display). Each app crate has a pure `translate` module (`apps/proto123-wasm/src/translate.rs` takes DOM-shaped primitives — no `web-sys` types; `apps/proto123-term/src/translate.rs` takes crossterm events) so both run natively in `cargo test` and must produce identical snapshots (FR-049, SC-012).
- **Performance**: deterministic assertions only in tests (evaluation counts, redraw region sizes). Timing evidence: `cargo run --release -p proto123-tui --example frame_budget` builds the reference sheet (10,000 cells, 5,000 formulas) and reports p50/p99 of input→buffer-ready time against `TestBackend`; the web check is manual with the browser DevTools Performance panel (see quickstart). Wall-clock is never asserted in CI (AGENTS rule 29).

---

All Technical Context items are resolved; no NEEDS CLARIFICATION remains.
