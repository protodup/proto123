# Classic123 Architecture

> **Last reviewed:** 2026-11-24 · **Last updated:** 2026-11-24 · **Change:** initial architecture baseline aligned with Spec Kit constitution and `AGENTS.md` · **Target:** OASIS ODF 1.3 (ISO/IEC 26300) + Classic Keyboard-First / Mouse-Enabled TUI Ergonomics

## Current state (2026-11-24)

What exists on `main` today. The numbered sections below are the foundational design plan; as implementation progresses, update this summary first—where the code differs from the original plan, this summary wins.

- **Workspace:** Planned L0–L6 layered Cargo workspace (`crates/classic123-*`, `apps/classic123-wasm`, `apps/classic123-cli`, `xtask/`). Layer boundaries are enforced by `cargo xtask layers` (`xtask/src/layers.rs`).
- **Rendering & UI:** Decided: single HTML5 `<canvas>` (2D or WebGL) rendering a character-cell TUI buffer produced entirely in Rust (`classic123-tui`) compiled to `wasm32-unknown-unknown`. Zero per-cell HTML DOM nodes; minimal JS/TS host in `web/`.
- **Engine:** Command-driven session (`classic123-engine`) where keyboard shortcuts, hierarchical `/` menus, mouse actions, and headless CLI invocations all dispatch typed `CommandId`s.
- **File Formats:** Pure-Rust ODF 1.3 (`.ods`) reader/writer (`classic123-ods`) with OpenFormula (`of:=...`) round-trip fidelity, plus RFC 4180 CSV (`classic123-csv`).
- **Deployment:** Runs 100% client-side as static assets (local dev server or multi-stage Docker container serving `application/wasm` and `application/vnd.oasis.opendocument.spreadsheet`), with remote URL `fetch()` loading and `PUT`/`POST`/download persistence hooks.
- **Not built yet:** All workspace crates are at pre-MVP scaffolding stage; implementation proceeds through GitHub Spec Kit specs under `.specify/specs/`.

---

# Design Plan (2026-11-24)

This document sets out:
- the architectural principles and historical spreadsheet traps we avoid from day one,
- the L0–L6 Cargo workspace layout and strict dependency rules,
- how the sparse grid, OpenFormula AST, and incremental DAG recalculation engine fit together,
- the data-driven TUI buffer, single-`<canvas>` presenter, and exact keyboard + mouse hit-testing contract,
- the ODF 1.3 (`.ods`) serialization architecture, thin web host, headless CLI, and Docker deployment model.

---

## 1. Goals and Principles

**Goals**
- An ultra-lightweight, sub-millisecond spreadsheet with classic terminal (TUI) clarity running 100% in the browser via **Rust + WebAssembly**, as well as locally and via **Docker**.
- **Full hybrid ergonomics:** effortless classic keyboard speed (`/` menu tree, direct cell entry, zero focus traps) combined with first-class mouse support on both the grid and menus.
- **100% OASIS ODF 1.3 (`.ods`) compatibility:** files created or edited by Classic123 open cleanly without warnings or formula loss in LibreOffice Calc, Collabora, and other ODF-compliant suites.
- **Swappable presenter:** the Rust core (`L0`–`L4`) knows nothing about the browser, Canvas, or terminal backend; the same engine drives the Web/Canvas TUI, a native terminal frontend (if added later), and the headless CLI.
- **Standalone reusable codecs:** `classic123-ods` and `classic123-formula` compile cleanly on native and `wasm32-unknown-unknown` with zero UI coupling.

**Principles**

1. **Engine-first, headless-first.** Every spreadsheet capability is exercisable without a browser or Canvas—from `cargo test`, `classic123-cli`, and scripted automation. The Canvas TUI is just one client of the engine.
2. **Everything is a command.** Every state-mutating action has a stable `CommandId` (e.g., `file.saveOds`, `range.copy`, `sheet.insertRow`, `cell.commitEdit`) with typed, serializable parameters. Keyboard shortcuts, the `/` menu bar, mouse gestures, undo/redo, and CLI commands all dispatch the exact same commands.
3. **Zero DOM per cell; data describes the TUI.** `classic123-tui` outputs a pure-data 2D character grid (`CellBuffer`), dirty-rectangle list, and hit-test layout map. The Canvas presenter simply paints monospace glyphs and box-drawing lines—never HTML `<div>` or `<table>` nodes.
4. **Thin JS host, 100% Rust logic.** All formula evaluation, sparse storage, ODF packaging, layout, and pixel-to-cell hit-testing live in Rust. `web/` exists strictly to boot WASM, forward raw browser input events, mirror active cell announcements to a single ARIA live region, and bridge `fetch()` / file download/upload.
5. **Sparse storage & incremental DAG recalculation.** Never allocate dense matrices for empty space, and never recalculate the whole workbook on a single edit. Only the dirty topological subgraph of dependent cells recomputes.
6. **Determinism & no-panic runtime.** The same workbook and command sequence yield identical values across native and `wasm32`. Non-test code must never panic on malformed formulas, corrupt `.ods` archives, or extreme coordinates; errors surface cleanly as typed spreadsheet errors (`#DIV/0!`, `#REF!`, `#NAME?`, `#VALUE!`, `#CIRC!`, `#ERR`).
7. **Pure Rust, minimal WASM footprint.** Avoid C/C++ dependencies and heavy runtime crates so the optimized `.wasm` artifact stays fast to download, compile, and instantiate.
8. **Clean-room & trademark safety.** Study public standards (OASIS ODF 1.3, OpenFormula 1.3, RFC 4180) and observable spreadsheet workflows only. Never copy or read GPL/LGPL/AGPL or proprietary spreadsheet code, and never use third-party trademarks as our own feature names.

### 1.1 Avoiding Historical Spreadsheet Traps

Most slow web spreadsheets and rigid legacy clones suffer from the same root cause: **UI, storage, and file-format shortcuts baked into the core as hard-coded assumptions.** Fixing them later forces a full rewrite.

**Rules we adopt from day one** (cheap now, prohibitively expensive later):

| Trap | Historical Failure Mode | Rule Adopted from Day One | Where |
|---|---|---|---|
| **DOM-per-cell UI** | Rendering thousands of `<td>`/`<div>` nodes triggers browser layout thrashing, memory bloat, and sluggish keystroke latency. | Single `<canvas>` presenter painting a Rust `CellBuffer` with dirty-rect diffing. Zero DOM elements for cells or menus. | `classic123-tui`, `apps/classic123-wasm`, `web/` |
| **Dense fixed arrays** | Legacy 2D arrays either hit tiny limits (256 × 8,192) or waste gigabytes of RAM for sparse sheets. | Sparse `HashMap<CellAddr, Cell>` backed by `u32` row/col indices (supporting full ODF/modern sheet extents up to 1,048,576 rows × 16,384 cols). | `classic123-coord`, `classic123-model` |
| **Full-sheet recalculation** | Re-evaluating every formula on every edit causes visible stalls once a workbook grows beyond a few hundred formulas. | Explicit dependency graph (DAG) with reverse edges, cycle detection (`#CIRC!`), and dirty-subgraph topological evaluation. | `classic123-calc` |
| **Lossy `.ods` export** | Treating ODF as a late "export filter" drops styles, sheet metadata, number formats, and produces non-standard formula strings. | ODF 1.3 & OpenFormula (`of:=...`) are first-class primitives. Package integrity (`mimetype` STORED first, manifest, XML namespaces) is tested on every CI run, and unknown ODF metadata/styles are preserved in pass-through bags where practical. | `classic123-ods`, `classic123-formula` |
| **Bolted-on mouse support** | Terminal apps often hard-code text layout and add mouse clicks later with brittle offset math that breaks on resize or scroll. | Unified `LayoutMap` emitted alongside `CellBuffer` from a single `CellMetrics` source of truth. Mouse hit-testing (grid selection, double-click edit, drag range, scroll, menu hover/click) uses the exact rects that were just drawn. | `classic123-tui` |
| **UI-coupled state** | Letting frontend event handlers mutate cells directly makes undo, CLI testing, and keyboard/mouse parity impossible to maintain. | All mutations flow through `classic123-engine` `CommandId`s and reversible `classic123-ops` deltas. | `classic123-ops`, `classic123-engine` |

The architectural litmus test: **adding a new OpenFormula function, a new TUI theme, or a native terminal frontend must touch only a leaf crate—never the core model or engine.**

---

## 2. Workspace Layout

```text
rusted-classic-123/
├─ Cargo.toml                     # [workspace], shared deps, workspace lints (unsafe_code = "forbid"), release/wasm profiles
├─ AGENTS.md                      # Operational rules for human and AI contributors
├─ ATTRIBUTION.md                 # Provenance and licences for fonts and test fixtures
├─ Dockerfile                     # Multi-stage static build (Rust+WASM -> minimal HTTP server with WASM/ODS MIME types)
├─ .specify/
│  ├─ memory/constitution.md      # GitHub Spec Kit architectural constitution
│  └─ specs/                      # Feature specifications, technical plans, and task breakdowns
├─ crates/
│  │  ── L0: Primitives (zero workspace deps, wasm-safe) ──
│  ├─ classic123-coord/           # CellAddr (col/row u32), CellRange, A1 & R1C1 parsing/formatting, sheet refs
│  ├─ classic123-value/           # CellValue (Empty, Number, Text, Bool, Date/Time via chrono, Error), number formatting
│  ├─ classic123-theme/           # TUI color palettes, high-contrast tokens (4.5:1 / 3:1), box-drawing glyphs, CellMetrics
│  │  ── L1: Document Model & Formula Language ──
│  ├─ classic123-model/           # Sparse Workbook, Sheet (HashMap<CellAddr, Cell>), CellStyle, col widths, row heights, ODF pass-through metadata
│  ├─ classic123-formula/         # Tokenizer, Pratt/recursive-descent parser, AST, OpenFormula (`of:=...`) canonical emitter & evaluator
│  │  ── L2: Calculation Engine & Reversible Ops ──
│  ├─ classic123-calc/            # Dependency graph (DAG), volatile tracking, cycle detection, incremental topological recalculation
│  ├─ classic123-ops/             # Atomic reversible workbook mutations (set cell, copy/move range, insert/delete row/col, formula ref shifting, undo/redo deltas)
│  │  ── L3: File Formats & ODF 1.3 Codecs ──
│  ├─ classic123-ods/             # Pure-Rust ODF 1.3 (.ods) ZIP/XML reader & writer (`mimetype` STORED first, content/styles/meta/manifest, OpenFormula)
│  ├─ classic123-csv/             # RFC 4180 CSV/TSV reader & writer with delimiter/encoding guards
│  │  ── L4: Engine, Modes & Command Registry ──
│  ├─ classic123-engine/          # Session, mode state machine (READY/EDIT/MENU/POINT/VALUE/LABEL), CommandId registry, menu_catalog.rs, viewport & selection state
│  │  ── L5: TUI Buffer, Layout & Hit-Testing ──
│  ├─ classic123-tui/             # Character CellBuffer, dirty-rect diff, `/` menu bar, formula bar, grid, status bar, exact pixel -> grid/menu hit-testing
│  │  ── Support ──
│  └─ classic123-testkit/         # Workbook builders, ODF 1.3 ZIP/XML structural validators, hostile-input generators
├─ apps/
│  ├─ classic123-wasm/            # wasm-bindgen bridge + Canvas 2D / WebGL glyph presenter + frame loop
│  └─ classic123-cli/             # Headless CLI (inspect, recalc, convert, validate .ods, scripted command runner)
├─ web/                           # Minimal static HTML/CSS/TS host (WASM boot, raw input forwarding, ARIA live region, remote fetch & PUT/POST hooks)
├─ xtask/                         # Workspace invariant checks (layers, menu-commands, wasm, ods-conformance, parity, docker-check)
├─ corpus/                        # Clean-room .ods conformance samples and hostile-input fixtures
└─ docs/
   ├─ architecture.md             # This document
   ├─ development.md              # Local toolchain, wasm-pack/trunk, and Docker workflow
   ├─ odf-conformance.md          # Supported ODF 1.3 features & OpenFormula function coverage
   ├─ parity-checklist.md         # Live matrix of commands across Keyboard, Mouse, and CLI
   └─ feature-names.md            # Trademark-safe user-visible feature terminology
```

---

## 3. Layering Rules and Dependency Boundaries

Dependencies flow strictly downward from `L6` to `L0`. Every new crate MUST be registered in `xtask/src/layers.rs` and verified by `cargo xtask layers`.

```text
L6  apps/classic123-wasm  |  apps/classic123-cli
             │                       │
L5  classic123-tui                   │
             │                       │
L4  ─────────┴─────── classic123-engine ───────┐
                        │           │          │
L3            classic123-ods   classic123-csv  │
                    │               │          │
L2         classic123-calc    classic123-ops   │
                    │               │          │
L1         classic123-model  classic123-formula│
                    │               │          │
L0  classic123-coord | classic123-value | classic123-theme
```

**Hard rules:**
1. **No upward or cyclic edges:** A lower layer MUST NEVER import a higher layer.
2. **WASM purity (`L0`–`L5`):** Every library crate from `L0` through `L5` MUST compile cleanly for `--target wasm32-unknown-unknown` with no `std::fs`, no blocking threads, and no OS-specific syscalls. File I/O in `classic123-ods` and `classic123-csv` operates on `Read + Seek` / `Write` or `&[u8]` / `Vec<u8>` buffers, leaving actual filesystem or `fetch()` transport to `L6`.
3. **Zero UI bleed:** `L0`–`L4` MUST NOT depend on `web-sys`, `wasm-bindgen`, Canvas types, or pixel coordinates. Pixel coordinates enter the system only at `L5` (`classic123-tui`) and `L6` (`apps/classic123-wasm`).
4. **No `unsafe` by default:** Workspace root enforces `unsafe_code = "forbid"`. Every crate inherits `[lints] workspace = true`.

---

## 4. Sparse Document Model, Formula AST & DAG Engine (`L0`–`L2`)

### 4.1 Sparse Workbook Model (`classic123-model`)
- A `Workbook` holds an ordered list of `Sheet`s, workbook metadata (`meta.xml` fields), a shared style table (`StyleId -> CellStyle`), and an optional opaque ODF extension bag for unknown attributes/elements preserved during round-trip.
- Each `Sheet` stores populated cells in a sparse `HashMap<CellAddr, Cell>` (or BTree/chunked sparse map for fast viewport range queries), plus sparse overrides for column widths (`HashMap<u32, u16>`) and row heights (`HashMap<u32, u16>`).
- Each `Cell` stores:
  - `raw_input: CellInput` (literal value, text label with alignment prefix `'`, `"`, `^`, or parsed formula AST + canonical OpenFormula string),
  - `cached_value: CellValue` (the latest evaluated result),
  - `style_id: StyleId`.
- Dates and durations use `chrono` types internally and map deterministically to ODF ISO 8601 values (`office:date-value`) and spreadsheet serial numbers when referenced in arithmetic.

### 4.2 Formula Engine (`classic123-formula`)
- **Single canonical AST:** User input (`=SUM(A1:A10)`, `@SUM(A1..A10)` classic shorthand if enabled, or imported `of:=[.A1]+[.A2]`) parses into one canonical AST (`Expr`).
- **OpenFormula Canonical Emitter:** When serializing to `.ods`, the AST emits strictly compliant OASIS OpenFormula (`of:=SUM([.A1:.A10])`) with proper sheet/cell bracket notation (`[.A1]`, `['Sheet 2'.B3]`).
- **Explicit Errors:** Unknown functions or type mismatches never silently evaluate to `0`; they propagate typed `CalcError` values (`#NAME?`, `#VALUE!`, `#REF!`, `#DIV/0!`, `#NUM!`, `#N/A`, `#CIRC!`).

### 4.3 Incremental Recalculation (`classic123-calc`)
- `DependencyGraph` tracks forward edges (`cell -> referenced cells/ranges`) and reverse edges (`cell/range -> dependent formulas`).
- On any cell mutation (`classic123-ops`):
  1. Update edges for the edited cell only.
  2. Collect reachable dependents via BFS/DFS into a dirty set.
  3. Topologically sort the dirty set (Kahn's algorithm); any cycle members receive `CalcError::Circular` (`#CIRC!`) without infinite recursion.
  4. Evaluate dirty cells in topological order and emit a list of `DirtyCell` notifications so `classic123-tui` only repaints changed screen regions.

---

## 5. Engine, Interaction Modes & Command Registry (`L4`)

`classic123-engine` owns the live `Session` and enforces that **every action is a command**.

### 5.1 Mode State Machine
Inspired by classic terminal spreadsheets, `SessionMode` makes keyboard and mouse behavior unambiguous and always visible in the top-right/status indicator:

- **`READY`:** Navigating the grid (arrows, PageUp/Down, Home/End, mouse click/drag/scroll). Typing a number or `=` immediately enters `VALUE`; typing a letter enters `LABEL`; pressing `/` opens `MENU`.
- **`VALUE` / `LABEL` / `EDIT`:** Editing the active cell in the top formula/input line. `Enter` dispatches `cell.commitEdit`; `Esc` cancels cleanly.
- **`POINT`:** While entering a formula, moving cursor keys or clicking/dragging cells with the mouse inserts live cell or range references (e.g., `B2:D8`) directly into the formula buffer.
- **`MENU`:** Navigating the two-line hierarchical classic command bar (e.g., `/File`, `/Range`, `/Worksheet`, `/Edit`, `/View`, `/Help`). Operable simultaneously via single-letter hotkeys, arrow keys + `Enter`, or mouse hover + single click. `Esc` steps back one menu level.
- **`DIALOG`:** Lightweight terminal prompt overlay (e.g., remote URL input, file save name, confirmation) with full keyboard and mouse click support.

### 5.2 Command Registry & Menu Parity
- Every action has a stable `CommandId` and `CommandSpec` (name, category, parameter schema, undoable flag).
- `menu_catalog.rs` defines the hierarchical `/` menu tree purely as data pointing to `CommandId`s.
- `cargo xtask menu-commands` fails CI if any menu item points to a missing command or unimplemented stub.
- `classic123-ops` records reversible `UndoEntry` deltas on every mutating command so Undo/Redo is O(changed cells), not a full workbook clone.

---

## 6. Data-Driven TUI Buffer, Canvas Presenter & Mouse Hit-Testing (`L5`–`L6`)

To achieve zero-DOM speed and pixel-accurate mouse interaction, rendering and hit-testing share a single layout pass inside `classic123-tui`:

```text
Browser DOM Event (keydown / pointerdown / pointermove / wheel / resize)
        │  (thin JS host forwards raw numbers/keys only)
        ▼
apps/classic123-wasm  ──►  classic123-tui::InputMapper (pixel x,y -> HitTarget via LayoutMap)
                                   │
                                   ▼
                           classic123-engine::dispatch(CommandId, Params)
                                   │
                                   ▼
                           classic123-tui::render(SessionView, Theme)
                             ├─► CellBuffer (2D grid of StyledGlyph) + DirtyRects
                             └─► LayoutMap (exact pixel/char rects for grid cells, headers, tabs, menu items, dialogs)
                                   │
                                   ▼
apps/classic123-wasm  ──►  Canvas 2D / WebGL Presenter (draws only dirty rects)
```

### 6.1 `CellBuffer` & Dirty-Rect Painting
- The viewport is divided into a fixed character grid `(cols, rows)` computed from `(canvas_width, canvas_height)` and `CellMetrics { char_width, char_height, dpr }`.
- Every frame produces a `CellBuffer` (flat `Vec<StyledGlyph>`). Diffing against `prev_buffer` yields minimal dirty runs/rects.
- The Canvas presenter in `apps/classic123-wasm` redraws **only modified cells**, maintaining consistent sub-16ms (60–120 FPS) frame times even on modest hardware.

### 6.2 First-Class Mouse Hit-Testing (`LayoutMap`)
Every rendered UI region registers its exact bounding box in `LayoutMap`:
- **Menu Bar & Submenus:** Hover updates `hovered_menu_item` (instant visual highlight); click dispatches the corresponding submenu transition or `CommandId`.
- **Formula Bar:** Click focuses `EDIT` mode and places the text caret at the clicked character column.
- **Grid Cells:**
  - Single click -> `nav.selectCell(addr)` (or inserts reference if in `POINT` mode).
  - Double click -> `cell.beginEdit(addr)`.
  - Click + drag -> updates active range selection (`nav.extendSelection(addr)`) with auto-scroll near viewport edges.
  - Scroll wheel (vertical + Shift/horizontal) -> `view.scroll(dx, dy)`.
- **Column/Row Headers & Sheet Tabs:** Click selects whole column/row or switches active sheet; dragging column divider resizes column width via `sheet.resizeColumn`.

### 6.3 Accessibility Without Cell DOM
- The `<canvas>` element stays focusable (`tabindex="0"`).
- Default TUI themes guarantee at least **4.5:1** text contrast and **3:1** border/focus contrast, using both symbols/brackets (`[A1]`, inverted video, explicit mode labels) and color—never color alone.
- On active cell change, mode change, or error status, `apps/classic123-wasm` pushes a concise string (e.g., `"B4, formula result 420.00, READY"`) to a single hidden `aria-live="polite"` element in `web/index.html`.

---

## 7. ODF 1.3 (`.ods`) Codec Architecture (`classic123-ods`)

`classic123-ods` guarantees **100% conformance with OASIS ODF 1.3 / ISO/IEC 26300** in pure Rust (`zip` + `quick-xml`, compiled to both native and `wasm32`):

1. **Strict Package Layout on Write:**
   - Entry #0 MUST be `mimetype` stored with compression method `STORED` (uncompressed), no extra ZIP fields, containing exact ASCII bytes `application/vnd.oasis.opendocument.spreadsheet`.
   - Required XML parts: `content.xml`, `styles.xml`, `meta.xml`, and `META-INF/manifest.xml` (declaring `manifest:version="1.3"` and all package files).
2. **Typed Value & Formula Fidelity:**
   - Every cell writes the exact `office:value-type` (`float`, `currency`, `percentage`, `date`, `time`, `boolean`, `string`), machine value attribute (`office:value` / `office:date-value` / `office:boolean-value`), cached display text (`<text:p>`), and—when applicable—`table:formula="of:=..."`.
   - Repeated empty cells/rows compress cleanly via `table:number-columns-repeated` and `table:number-rows-repeated` so sparse workbooks produce tiny `.ods` files.
3. **Bounded Hostile-Input Reading:**
   - Enforces decompression size caps (zip-bomb protection), XML nesting depth limits, and coordinate bounds so malicious or corrupted `.ods` files return typed errors instead of panicking or exhausting browser memory.
4. **Verification (`cargo xtask ods-conformance`):**
   - Validates ZIP entry order/compression, XML well-formedness, namespace URIs, OpenFormula syntax, and full `Workbook -> .ods bytes -> Workbook -> recalculate` round-trip invariants across `corpus/`.

---

## 8. Thin Web Host, Remote Files & Docker Deployment (`web/`, `Dockerfile`)

### 8.1 Thin Web Host Contract (`web/`)
`web/` contains only static HTML, minimal CSS (fullscreen canvas reset + focus ring), and a small TypeScript/JS bootstrapper (~150–250 lines):
- Instantiates the `.wasm` module via streaming compilation (`WebAssembly.instantiateStreaming`).
- Attaches passive/non-blocking listeners for keyboard, pointer/mouse, wheel, clipboard (`copy`/`paste`), and `ResizeObserver`, forwarding primitive values into WASM.
- **Remote File Loading:** Supports URL parameter or config hook (e.g., `?file=/sheets/budget.ods`) that executes `fetch(url) -> ArrayBuffer -> Uint8Array` and passes bytes directly into `classic123_wasm::load_ods_bytes(...)`.
- **Persistence Hooks:** Exports `.ods` bytes from WASM either as a local browser download (`Blob` + `<a download>`) or via a configurable async `PUT`/`POST` callback to the user's server.

### 8.2 Local Execution & Multi-Stage Docker Packaging
- **Local:** `trunk serve` (or static build served from `dist/`) runs directly on localhost with zero backend dependencies.
- **Docker:** A multi-stage `Dockerfile`:
  1. *Builder stage:* Rust + `wasm32-unknown-unknown` + `wasm-pack`/`trunk` + `wasm-opt -Oz` builds the minified static bundle (`dist/`).
  2. *Runtime stage:* Minimal static web server (e.g., Alpine Nginx or Caddy, distroless/scratch footprint) serving `dist/` with explicit headers:
     - `.wasm` -> `Content-Type: application/wasm`
     - `.ods` -> `Content-Type: application/vnd.oasis.opendocument.spreadsheet`
     - Pre-compressed `.br` / `.gz` static assets enabled for instant cold loads.
- `cargo xtask docker-check` verifies container build and response MIME types automatically.

---

## 9. Headless CLI, Testing & Automated Checks

### 9.1 `classic123-cli`
Provides a native headless interface to the exact same `L0`–`L4` engine:
- `classic123-cli inspect <file.ods>` — prints sheets, dimensions, formula counts, and ODF structural diagnostics.
- `classic123-cli recalc <in.ods> -o <out.ods>` — loads, recalculates DAG, and writes a valid `.ods`.
- `classic123-cli exec <script.json> --read-root <dir> --write-root <dir>` — runs scripted `CommandId` sequences with strict path-root sandboxing for automated regression testing.

### 9.2 Verification Suite
| Check | Command | What It Guards |
|---|---|---|
| Unit & integration tests | `cargo test -p <crate>` | Formula math, DAG topo-sort, ODF round-trip, pixel-to-cell hit-testing |
| Strict lints | `cargo clippy --workspace --all-targets -- -D warnings` | Zero warnings, no accidental panics or `unsafe` |
| Architecture graph | `cargo xtask layers` | Enforces L0–L6 dependency DAG |
| Menu/command parity | `cargo xtask menu-commands` | Every `/` menu item resolves to a live `CommandId` |
| WASM target & size | `cargo xtask wasm` | Compiles `wasm32-unknown-unknown` and checks `.wasm` size budget |
| ODF 1.3 conformance | `cargo xtask ods-conformance` | Validates ZIP `mimetype` first/STORED, XML structure, OpenFormula round-trips |
| Hostile input sweep | `cargo test -p classic123-engine --test panic_hunt -- --ignored` | Corrupt ZIPs, cyclic formulas, extreme coords never panic |
| Docker & MIME check | `cargo xtask docker-check` | Verifies static container serves `application/wasm` and `.ods` cleanly |

---

## 10. Phased Delivery Roadmap (Spec Kit Aligned)

Each phase maps to one or more GitHub Spec Kit specifications under `.specify/specs/`:

- **Phase 0 — Workspace & Guardrails:** Cargo workspace, `unsafe_code = "forbid"`, `xtask` (`layers`, `wasm`, `menu-commands`), CI skeleton, and multi-stage `Dockerfile`.
- **Phase 1 — Core Engine (`L0`–`L2`):** `coord`, `value` (`chrono` dates), `model` (sparse grid), `formula` (lexer/parser/OpenFormula evaluator for arithmetic + core functions), `calc` (DAG + cycle detection), and `ops` (undo/redo).
- **Phase 2 — Interactive TUI-on-Canvas MVP (`L4`–`L6`):** `engine` modes (`READY`, `EDIT`, `MENU`, `POINT`), `tui` `CellBuffer` + dirty rects + `LayoutMap` mouse hit-testing, `apps/classic123-wasm` Canvas presenter, and minimal `web/` host with full keyboard + mouse support.
- **Phase 3 — 100% ODF 1.3 & Server I/O (`L3` + `web/`):** `classic123-ods` reader/writer, `classic123-csv`, `xtask ods-conformance` corpus suite, remote `fetch()` loader, local download, and `PUT`/`POST` save hooks.
- **Phase 4 — Polish, CLI & Parity Expansion:** `classic123-cli`, expanded OpenFormula library (lookup, financial, date, logical, string functions), cell number formatting/styles, column resizing by mouse drag, and WASM bundle size optimization.