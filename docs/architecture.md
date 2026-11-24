# Classic123 Architecture

> **Last reviewed:** 2026-11-24 · **Last updated:** 2026-11-24 · **Change:** aligned with constitution v1.1.0 (native terminal target, Ratatui + Ratzilla/crossterm, mouse on both targets, `.xlsx`/`.csv` import-only) · **Target:** OASIS ODF 1.3 (ISO/IEC 26300) + Classic Keyboard-First / Mouse-Enabled TUI Ergonomics

## Current state (2026-11-24)

What exists on `main` today. The numbered sections below are the foundational design plan; as implementation progresses, update this summary first—where the code differs from the original plan, this summary wins.

- **Workspace:** Planned L0–L6 layered Cargo workspace (`crates/classic123-*`, `apps/classic123-wasm`, `apps/classic123-cli`, `xtask/`). Layer boundaries are enforced by `cargo xtask layers` (`xtask/src/layers.rs`).
- **Targets:** One shared Rust core (`L0`–`L5`) drives two front ends: the **web target** (`apps/classic123-wasm`, `wasm32-unknown-unknown`, single `<canvas>`) and the **native terminal target** (`apps/classic123-term`, Linux/macOS/Windows). A headless CLI (`apps/classic123-cli`) exercises the same engine without any UI.
- **Rendering & UI:** Decided: TUI layout is written once with **Ratatui** in `classic123-tui`; the web target draws it with the **Ratzilla** Canvas/WebGL backend and the native target with the **crossterm** backend. Zero per-cell HTML DOM nodes; minimal JS/TS host in `web/`. A custom Canvas 2D renderer is allowed only if a feature plan shows Ratzilla cannot meet the performance budget, and it must consume the same Ratatui layout.
- **Mouse:** Full mouse support on both targets—Canvas pointer events on the web, terminal mouse reporting on native—interpreted by one shared `InputMapper` and `LayoutMap`.
- **Engine:** Command-driven session (`classic123-engine`) where keyboard shortcuts, hierarchical `/` menus, mouse actions, and headless CLI invocations all dispatch typed `CommandId`s.
- **File Formats:** Pure-Rust ODF 1.3 (`.ods`) reader/writer (`classic123-ods`) with OpenFormula (`of:=...`) round-trip fidelity; `.xlsx` (`classic123-xlsx`) and RFC 4180 CSV (`classic123-csv`) are import-only. Every file written is `.ods`.
- **Deployment:** The web target runs 100% client-side as static assets (local dev server, offline, or multi-stage Docker container serving `application/wasm` and `application/vnd.oasis.opendocument.spreadsheet`), with remote URL `fetch()` loading and download / opt-in `PUT`/`POST` persistence. The native target is a single binary that opens local files or HTTP URLs, saves to the local filesystem, and offers the same opt-in `PUT`/`POST` hook.
- **Not built yet:** All workspace crates are at pre-MVP scaffolding stage; implementation proceeds through GitHub Spec Kit specs under `.specify/specs/`.

---

# Design Plan (2026-11-24)

This document sets out:
- the architectural principles and historical spreadsheet traps we avoid from day one,
- the L0–L6 Cargo workspace layout and strict dependency rules,
- how the sparse grid, OpenFormula AST, and incremental DAG recalculation engine fit together,
- the shared Ratatui layout, the two backends (Ratzilla on a single `<canvas>`, crossterm in the native terminal), and the exact keyboard + mouse hit-testing contract,
- the ODF 1.3 (`.ods`) serialization and `.xlsx`/`.csv` import architecture, thin web host, native terminal front end, headless CLI, and Docker deployment model.

---

## 1. Goals and Principles

**Goals**
- An ultra-lightweight spreadsheet with classic terminal (TUI) clarity running 100% in the browser via **Rust + WebAssembly**, locally as a **native terminal application**, and via **Docker**, with input-to-frame latency under 16 ms on every target.
- **Full hybrid ergonomics:** effortless classic keyboard speed (`/` menu tree, direct cell entry, zero focus traps) combined with first-class mouse support on the grid, menus, and dialogs—in the browser and in the terminal.
- **100% OASIS ODF 1.3 (`.ods`) compatibility:** files created or edited by Classic123 open cleanly without warnings or formula loss in LibreOffice Calc, Collabora, and other ODF-compliant suites.
- **Shared core, swappable backend:** the Rust core (`L0`–`L5`) knows nothing about the browser, Canvas, terminal, filesystem, or network; the same engine and Ratatui layout drive the web target, the native terminal target, and (without layout) the headless CLI. A feature behaves identically on both targets unless its plan documents a difference.
- **Standalone reusable codecs:** `classic123-ods` and `classic123-formula` compile cleanly on native and `wasm32-unknown-unknown` with zero UI coupling.

**Principles**

1. **Engine-first, headless-first.** Every spreadsheet capability is exercisable without a browser, Canvas, or terminal—from `cargo test`, `classic123-cli`, and scripted automation. The web and native TUIs are just two clients of the engine.
2. **Everything is a command.** Every state-mutating action has a stable `CommandId` (e.g., `file.saveOds`, `range.copy`, `sheet.insertRow`, `cell.commitEdit`) with typed, serializable parameters. Keyboard shortcuts, the `/` menu bar, mouse gestures, undo/redo, and CLI commands all dispatch the exact same commands.
3. **Zero DOM per cell; data describes the TUI.** `classic123-tui` renders Ratatui widgets into a pure-data character buffer (`ratatui::buffer::Buffer`) plus a hit-test `LayoutMap`. Ratatui's double-buffer diff sends only changed cells to the backend: Ratzilla paints them as monospace glyphs on one `<canvas>`—never HTML `<div>` or `<table>` nodes—and crossterm writes them as terminal escape sequences.
4. **Thin hosts, 100% Rust logic.** All formula evaluation, sparse storage, file codecs, layout, and hit-testing live in the shared Rust core. `web/` exists strictly to boot WASM, forward raw browser input events, mirror active cell announcements to a single ARIA live region, and bridge `fetch()` / file download/upload. `apps/classic123-term` exists strictly to run the terminal event loop, forward raw crossterm events, and perform filesystem and HTTP I/O.
5. **Sparse storage & incremental DAG recalculation.** Never allocate dense matrices for empty space, and never recalculate the whole workbook on a single edit. Only the dirty topological subgraph of dependent cells recomputes.
6. **Determinism & no-panic runtime.** The same workbook and command sequence yield identical values across native and `wasm32`. Non-test code must never panic on malformed formulas, corrupt `.ods` archives, or extreme coordinates; errors surface cleanly as typed spreadsheet errors (`#DIV/0!`, `#REF!`, `#NAME?`, `#VALUE!`, `#CIRC!`, `#ERR`).
7. **Pure Rust, minimal WASM footprint.** Avoid C/C++ dependencies and heavy runtime crates so the optimized `.wasm` artifact stays fast to download, compile, and instantiate.
8. **Clean-room & trademark safety.** Study public standards (OASIS ODF 1.3, OpenFormula 1.3, RFC 4180) and observable spreadsheet workflows only. Never copy or read GPL/LGPL/AGPL or proprietary spreadsheet code, and never use third-party trademarks as our own feature names.

### 1.1 Avoiding Historical Spreadsheet Traps

Most slow web spreadsheets and rigid legacy clones suffer from the same root cause: **UI, storage, and file-format shortcuts baked into the core as hard-coded assumptions.** Fixing them later forces a full rewrite.

**Rules we adopt from day one** (cheap now, prohibitively expensive later):

| Trap | Historical Failure Mode | Rule Adopted from Day One | Where |
|---|---|---|---|
| **DOM-per-cell UI** | Rendering thousands of `<td>`/`<div>` nodes triggers browser layout thrashing, memory bloat, and sluggish keystroke latency. | Single `<canvas>` drawn by the Ratzilla backend from a Ratatui buffer with diffed redraws. Zero DOM elements for cells or menus. | `classic123-tui`, `apps/classic123-wasm`, `web/` |
| **Dense fixed arrays** | Legacy 2D arrays either hit tiny limits (256 × 8,192) or waste gigabytes of RAM for sparse sheets. | Sparse `HashMap<CellAddr, Cell>` backed by `u32` row/col indices (supporting full ODF/modern sheet extents up to 1,048,576 rows × 16,384 cols). | `classic123-coord`, `classic123-model` |
| **Full-sheet recalculation** | Re-evaluating every formula on every edit causes visible stalls once a workbook grows beyond a few hundred formulas. | Explicit dependency graph (DAG) with reverse edges, cycle detection (`#CIRC!`), and dirty-subgraph topological evaluation. | `classic123-calc` |
| **Lossy `.ods` export** | Treating ODF as a late "export filter" drops styles, sheet metadata, number formats, and produces non-standard formula strings. | ODF 1.3 & OpenFormula (`of:=...`) are first-class primitives. Package integrity (`mimetype` STORED first, manifest, XML namespaces) is tested on every CI run, and unknown ODF metadata/styles are preserved in pass-through bags where practical. | `classic123-ods`, `classic123-formula` |
| **Bolted-on mouse support** | Terminal apps often hard-code text layout and add mouse clicks later with brittle offset math that breaks on resize or scroll. | Unified `LayoutMap` emitted by the same Ratatui layout pass that fills the buffer. Mouse hit-testing (grid selection, double-click edit, drag range, scroll, menu hover/click) uses the exact rects that were just drawn, on the web and in the native terminal. Gesture interpretation lives in one shared `InputMapper`. | `classic123-tui` |
| **Diverging front ends** | Web and desktop builds that each grow their own rules drift apart and double the test surface. | Both targets are thin adapters over the same `L0`–`L5` core; only event capture, the drawing backend, and I/O differ. | `apps/classic123-wasm`, `apps/classic123-term` |
| **UI-coupled state** | Letting frontend event handlers mutate cells directly makes undo, CLI testing, and keyboard/mouse parity impossible to maintain. | All mutations flow through `classic123-engine` `CommandId`s and reversible `classic123-ops` deltas. | `classic123-ops`, `classic123-engine` |

The architectural litmus test: **adding a new OpenFormula function, a new TUI theme, or a new UI feature must not require target-specific logic—the web and native front ends change only when event capture, drawing, or I/O changes.**

---

## 2. Workspace Layout

```text
classic123/
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
│  ├─ classic123-xlsx/            # .xlsx import only (sheets, typed values, formulas -> canonical AST); never written
│  ├─ classic123-csv/             # RFC 4180 CSV/TSV import only, with delimiter/encoding guards
│  │  ── L4: Engine, Modes & Command Registry ──
│  ├─ classic123-engine/          # Session, mode state machine (READY/EDIT/MENU/POINT/VALUE/LABEL), CommandId registry, menu_catalog.rs, viewport & selection state
│  │  ── L5: Shared TUI Layout & Input Mapping (Ratatui, backend-agnostic) ──
│  ├─ classic123-tui/             # Ratatui widgets (menu bar, formula bar, grid, tabs, dialogs, status bar), LayoutMap, pixel -> cell mapping, InputMapper (keys, click, double-click, drag, wheel, hover)
│  │  ── Support ──
│  └─ classic123-testkit/         # Workbook builders, ODF 1.3 ZIP/XML structural validators, hostile-input generators
├─ apps/
│  ├─ classic123-wasm/            # wasm-bindgen bridge + Ratzilla Canvas/WebGL backend + frame loop (web target)
│  ├─ classic123-term/            # Native terminal app: crossterm backend, event loop, mouse capture, filesystem & HTTP I/O (native target)
│  └─ classic123-cli/             # Headless CLI (inspect, recalc, convert .xlsx/.csv -> .ods, validate .ods, scripted command runner)
├─ web/                           # Minimal static HTML/CSS/TS host (WASM boot, raw input forwarding, ARIA live region, remote fetch & PUT/POST hooks)
├─ xtask/                         # Workspace invariant checks (layers, menu-commands, wasm, ods-conformance, parity, docker-check)
├─ corpus/                        # Clean-room .ods conformance samples and hostile-input fixtures
└─ docs/
   ├─ architecture.md             # This document
   ├─ development.md              # Local toolchain, wasm-pack/trunk, native terminal build, and Docker workflow
   ├─ interaction-spec.md         # Modes, keyboard map, `/` menu tree, mouse contract (web + native)
   ├─ odf-conformance.md          # Supported ODF 1.3 features & OpenFormula function coverage
   ├─ parity-checklist.md         # Live matrix of commands across Keyboard, Mouse, and CLI
   └─ feature-names.md            # Trademark-safe user-visible feature terminology
```

---

## 3. Layering Rules and Dependency Boundaries

Dependencies flow strictly downward from `L6` to `L0`. Every new crate MUST be registered in `xtask/src/layers.rs` and verified by `cargo xtask layers`.

```text
L6  apps/classic123-wasm  |  apps/classic123-term  |  apps/classic123-cli
      (Ratzilla)         │     (crossterm)        │
             └───────────┬──────────┘             │
L5               classic123-tui (Ratatui)         │
                         │                        │
L4  ─────────────────────┴─ classic123-engine ────┘─────┐
                        │           │                   │
L3   classic123-ods  classic123-xlsx  classic123-csv    │
                    │               │          │
L2         classic123-calc    classic123-ops   │
                    │               │          │
L1         classic123-model  classic123-formula│
                    │               │          │
L0  classic123-coord | classic123-value | classic123-theme
```

**Hard rules:**
1. **No upward or cyclic edges:** A lower layer MUST NEVER import a higher layer.
2. **Shared-core purity (`L0`–`L5`):** Every library crate from `L0` through `L5` MUST compile unchanged for both `wasm32-unknown-unknown` and native targets, with no `std::fs`, no network, no blocking threads, and no OS-specific syscalls. File codecs (`classic123-ods`, `classic123-xlsx`, `classic123-csv`) operate on `Read + Seek` / `Write` or `&[u8]` / `Vec<u8>` buffers, leaving filesystem, HTTP, and `fetch()` transport to `L6`.
3. **Zero UI bleed:** `L0`–`L4` MUST NOT depend on Ratatui, `web-sys`, `wasm-bindgen`, crossterm, Canvas types, or pixel coordinates. `L5` depends on `ratatui` only (no backend crate). Backends are chosen in `L6`: Ratzilla in `apps/classic123-wasm`, crossterm in `apps/classic123-term`. Canvas pixel coordinates enter the system only through the `L5` pixel → cell mapping called by `apps/classic123-wasm`; the native terminal already reports TUI cell coordinates.
4. **Native-only dependencies stay in `L6`:** crossterm, the HTTP client, and filesystem access are allowed only in `apps/classic123-term` and `apps/classic123-cli`, and MUST also respect the minimal-dependency and licence rules.
5. **No `unsafe` by default:** Workspace root enforces `unsafe_code = "forbid"`. Every crate inherits `[lints] workspace = true`.

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
  4. Evaluate dirty cells in topological order and emit a list of `DirtyCell` notifications so only changed screen regions are redrawn.

---

## 5. Engine, Interaction Modes & Command Registry (`L4`)

`classic123-engine` owns the live `Session` and enforces that **every action is a command**.

### 5.1 Mode State Machine
Inspired by classic terminal spreadsheets, `SessionMode` makes keyboard and mouse behavior unambiguous and always visible in the top-right/status indicator:

- **`READY`:** Navigating the grid (arrows, PageUp/Down, Home/End, mouse click/drag/scroll). Typing a number or `=` immediately enters `VALUE`; typing a letter enters `LABEL`; pressing `/` opens `MENU`.
- **`VALUE` / `LABEL` / `EDIT`:** Editing the active cell in the top formula/input line. `Enter` dispatches `cell.commitEdit`; `Esc` cancels cleanly.
- **`POINT`:** While entering a formula, moving cursor keys or clicking/dragging cells with the mouse inserts live cell or range references (e.g., `B2:D8`) directly into the formula buffer.
- **`MENU`:** Navigating the two-line hierarchical classic command bar (e.g., `/File`, `/Range`, `/Worksheet`, `/Edit`, `/View`, `/Help`). Operable simultaneously via single-letter hotkeys, arrow keys + `Enter`, or mouse hover + single click. `Esc` steps back one menu level.
- **`DIALOG`:** Lightweight TUI prompt overlay (e.g., remote URL input, file path / save name, confirmation) with full keyboard and mouse click support on both targets.

### 5.2 Command Registry & Menu Parity
- Every action has a stable `CommandId` and `CommandSpec` (name, category, parameter schema, undoable flag).
- `menu_catalog.rs` defines the hierarchical `/` menu tree purely as data pointing to `CommandId`s.
- `cargo xtask menu-commands` fails CI if any menu item points to a missing command or unimplemented stub.
- `classic123-ops` records reversible `UndoEntry` deltas on every mutating command so Undo/Redo is O(changed cells), not a full workbook clone.

---

## 6. Shared Ratatui Layout, Target Backends & Mouse Hit-Testing (`L5`–`L6`)

Rendering and hit-testing share a single layout pass inside `classic123-tui`, which is identical on both targets. Only event capture and the drawing backend differ:

```text
 Web target (apps/classic123-wasm)                Native target (apps/classic123-term)
 Browser event (keydown / pointer / wheel /       crossterm event (Key / Mouse{Down,Up,Drag,
 resize) forwarded raw by the thin JS host        Moved,ScrollUp/Down/Left/Right} / Resize)
        │                                                  │
        ▼                                                  │  (already in TUI cell units)
 classic123-tui::pixel_to_cell(x, y, CellMetrics)          │
        │  (Canvas pixels -> TUI cell (col, row))          │
        └──────────────────────┬───────────────────────────┘
                               ▼
        classic123-tui::InputMapper  (raw, timestamped events -> gestures:
                               │      click, double-click, drag, wheel, hover)
                               ▼      + LayoutMap lookup (TUI cell -> HitTarget)
        classic123-engine::dispatch(CommandId, Params)
                               │
                               ▼
        classic123-tui::render(SessionView, Theme)  (Ratatui widgets)
          ├─► ratatui::buffer::Buffer  (diffed by Ratatui; only changed cells redrawn)
          └─► LayoutMap (exact TUI cell rects for grid cells, headers, tabs, menu items, dialogs)
                               │
               ┌───────────────┴────────────────┐
               ▼                                ▼
     Ratzilla backend: paints glyphs     crossterm backend: writes escape
     on a single <canvas> (Canvas/WebGL) sequences to the terminal
```

### 6.1 Buffer & Diffed Redraws
- The viewport is a fixed character grid `(cols, rows)`. On the web it is computed from `(canvas_width, canvas_height)` and `CellMetrics { char_width, char_height, dpr }`; on native it is the terminal size reported by crossterm.
- Every frame renders Ratatui widgets into a `Buffer`. Ratatui diffs it against the previous frame so the backend redraws **only modified cells**; full redraws happen only on resize and theme change.
- Input-to-frame latency MUST stay under **16 ms** on both targets for the reference workloads defined in each feature plan.
- **Renderer choice:** Ratzilla is the default web backend. A custom Rust Canvas 2D character-grid renderer may replace it only if the first rendering feature plan shows Ratzilla cannot meet the performance budget; it MUST still consume the same Ratatui buffer and layout. The decision is recorded in that plan.

### 6.2 First-Class Mouse on Both Targets
- **Web:** the thin JS host forwards raw `pointerdown` / `pointermove` / `pointerup` and `wheel` (both axes) events with their timestamps; it never interprets them. `classic123-tui::pixel_to_cell` converts Canvas pixels to TUI cells, accounting for device pixel ratio, font metrics, and scroll offset.
- **Native terminal:** on startup `apps/classic123-term` enables raw mode, the alternate screen, and crossterm mouse capture (SGR extended reporting with button, drag, motion, and wheel events). A guard and panic hook restore the terminal (disable mouse capture, leave the alternate screen, disable raw mode) on every exit path. Terminals without mouse reporting fall back to complete keyboard operation.
- **Shared gesture logic:** terminals and browsers report presses, not semantic double-clicks, so `InputMapper` derives double-click (same cell, within a configurable interval), drag start/extend/end, and hover changes from raw timestamped events. The same unit tests cover both targets.

### 6.3 `LayoutMap` Hit-Testing
Every rendered UI region registers its exact TUI cell rectangle in `LayoutMap`:
- **Menu Bar & Submenus:** Hover updates `hovered_menu_item` (instant visual highlight); click dispatches the corresponding submenu transition or `CommandId`.
- **Formula Bar:** Click focuses `EDIT` mode and places the text caret at the clicked character column.
- **Grid Cells:**
  - Single click -> `nav.selectCell(addr)` (or inserts reference if in `POINT` mode).
  - Double click -> `cell.beginEdit(addr)`.
  - Click + drag -> updates active range selection (`nav.extendSelection(addr)`) with auto-scroll near viewport edges.
  - Wheel (vertical; horizontal via trackpad/tilt wheel or `Shift` + wheel) -> `view.scroll(dx, dy)`.
- **Column/Row Headers & Sheet Tabs:** Click selects whole column/row or switches active sheet; dragging column divider resizes column width via `sheet.resizeColumn`.
- **Dialogs:** Hover highlights `[OK]` / `[Cancel]`; click on an input field positions the caret.

Both mapping steps (`pixel_to_cell` and `LayoutMap` lookup) are pure Rust functions with unit tests.

### 6.4 Accessibility
- Default TUI themes guarantee at least **4.5:1** text contrast and **3:1** border/focus contrast, using both symbols/brackets (`[A1]`, inverted video, explicit mode labels) and color—never color alone. Native themes also provide a monochrome/ASCII fallback for limited terminals.
- **Web:** the `<canvas>` element stays focusable (`tabindex="0"`). On active cell change, mode change, or error status, `apps/classic123-wasm` pushes a concise string (e.g., `"B4, formula result 420.00, READY"`) to a single hidden `aria-live="polite"` element in `web/index.html`.
- **Native:** the terminal shows the same information in the control line and status bar as plain text, readable by terminal screen readers.

---

## 7. ODF 1.3 (`.ods`) Codec Architecture (`classic123-ods`)

Every file Classic123 writes is `.ods`. `classic123-ods` guarantees **100% conformance with OASIS ODF 1.3 / ISO/IEC 26300** in pure Rust (`zip` + `quick-xml`, or `spreadsheet-ods` if a plan adopts it; compiled to both native and `wasm32`):

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
5. **Import-Only Formats:**
   - `classic123-xlsx` reads `.xlsx` sheets, typed values, and formulas (translated to the canonical AST; untranslatable formulas keep their cached value and surface an explicit diagnostic). `classic123-csv` reads RFC 4180 CSV/TSV. Both apply the same zip-bomb, XML-depth, and coordinate guards as `classic123-ods`.
   - Saving an imported workbook always produces `.ods`; no `.xlsx` or `.csv` file is ever written.

---

## 8. Thin Hosts, Remote Files & Deployment (`web/`, `apps/classic123-term`, `Dockerfile`)

### 8.1 Thin Web Host Contract (`web/`)
`web/` contains only static HTML, minimal CSS (fullscreen canvas reset + focus ring), and a small TypeScript/JS bootstrapper (~150–250 lines):
- Instantiates the `.wasm` module via streaming compilation (`WebAssembly.instantiateStreaming`).
- Attaches passive/non-blocking listeners for keyboard, pointer/mouse, wheel, clipboard (`copy`/`paste`), and `ResizeObserver`, forwarding primitive values into WASM.
- **Remote File Loading:** Supports URL parameter or config hook (e.g., `?file=/sheets/budget.ods`) that executes `fetch(url) -> ArrayBuffer -> Uint8Array` and passes bytes directly into `classic123_wasm::load_bytes(bytes, name)`, which detects `.ods`, `.xlsx`, or `.csv` in Rust.
- **Persistence Hooks:** Exports `.ods` bytes from WASM either as a local browser download (`Blob` + `<a download>`) or via an opt-in, configured async `PUT`/`POST` callback to the user's server. No upload happens unless configured.
- **Failure handling:** network, CORS, unsupported, or corrupt files are reported in the status bar and never replace the current workbook.

### 8.2 Native Terminal Front End (`apps/classic123-term`)
A single native binary for Linux, macOS, and Windows, built from the same core:
- **Event loop:** reads crossterm key, mouse, paste, and resize events, forwards them to `classic123-tui::InputMapper`, and draws frames through Ratatui's crossterm backend.
- **Terminal lifecycle:** raw mode, alternate screen, and mouse capture are enabled on start and always restored on exit, error, or panic.
- **Local files:** `classic123-term <path>` and `/File/Open` read `.ods`, `.xlsx`, or `.csv` from the filesystem; `/File/Save ODS` (`Ctrl+S`) writes `.ods` to the local filesystem (atomic write via temporary file + rename, so a failed save never corrupts the original).
- **Remote files:** `classic123-term <https-url>` and `/File/Fetch URL` download through a minimal pure-Rust HTTP client (TLS via `rustls`), with the same size limits as the web target.
- **Server persistence:** the same opt-in `PUT`/`POST` hook, configured by command-line flag or config file; no upload happens unless configured.
- **Failure handling:** I/O, network, unsupported, or corrupt files are reported in the status bar without losing the current workbook.
- **No spreadsheet rules:** the app only translates terminal events and performs I/O; everything else lives in the shared core.

### 8.3 Local Execution & Multi-Stage Docker Packaging
- **Native:** `cargo run --release -p classic123-term -- budget.ods` (or the installed `classic123-term` binary).
- **Local web:** `trunk serve` (or static build served from `dist/`) runs directly on localhost, including offline, with zero backend dependencies.
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
Provides a native headless interface (no TUI) to the exact same `L0`–`L4` engine:
- `classic123-cli inspect <file.ods>` — prints sheets, dimensions, formula counts, and ODF structural diagnostics.
- `classic123-cli recalc <in.ods> -o <out.ods>` — loads, recalculates DAG, and writes a valid `.ods`.
- `classic123-cli convert <in.xlsx|in.csv> -o <out.ods>` — imports and writes a valid `.ods` (the only output format).
- `classic123-cli exec <script.json> --read-root <dir> --write-root <dir>` — runs scripted `CommandId` sequences with strict path-root sandboxing for automated regression testing.

### 9.2 Verification Suite
| Check | Command | What It Guards |
|---|---|---|
| Unit & integration tests | `cargo test -p <crate>` | Formula math, DAG topo-sort, ODF round-trip, pixel-to-cell and cell-to-element hit-testing, mouse gestures |
| Strict lints | `cargo clippy --workspace --all-targets -- -D warnings` | Zero warnings, no accidental panics or `unsafe` |
| Architecture graph | `cargo xtask layers` | Enforces L0–L6 dependency DAG |
| Menu/command parity | `cargo xtask menu-commands` | Every `/` menu item resolves to a live `CommandId` |
| WASM target & size | `cargo xtask wasm` | Compiles `wasm32-unknown-unknown`, runs `wasm-opt`, and reports compressed `.wasm` size (numeric ceiling adopted once a baseline exists) |
| Native target | `cargo build -p classic123-term` | Native terminal app builds on Linux, macOS, and Windows |
| ODF 1.3 conformance | `cargo xtask ods-conformance` | Validates ZIP `mimetype` first/STORED, XML structure, OpenFormula round-trips |
| Hostile input sweep | `cargo test -p classic123-engine --test panic_hunt -- --ignored` | Corrupt ZIPs, cyclic formulas, extreme coords never panic |
| Docker & MIME check | `cargo xtask docker-check` | Verifies static container serves `application/wasm` and `.ods` cleanly |

---

## 10. Phased Delivery Roadmap (Spec Kit Aligned)

Each phase maps to one or more GitHub Spec Kit specifications under `.specify/specs/`:

- **Phase 0 — Workspace & Guardrails:** Cargo workspace, `unsafe_code = "forbid"`, `xtask` (`layers`, `wasm`, `menu-commands`), CI skeleton, and multi-stage `Dockerfile`.
- **Phase 1 — Core Engine (`L0`–`L2`):** `coord`, `value` (`chrono` dates), `model` (sparse grid), `formula` (lexer/parser/OpenFormula evaluator for arithmetic + core functions), `calc` (DAG + cycle detection), and `ops` (undo/redo).
- **Phase 2 — Interactive TUI MVP on both targets (`L4`–`L6`):** `engine` modes (`READY`, `EDIT`, `MENU`, `POINT`), `tui` Ratatui layout + `LayoutMap` + `InputMapper`, `apps/classic123-wasm` with Ratzilla and the minimal `web/` host, and `apps/classic123-term` with crossterm—full keyboard + mouse support on both. The first rendering plan records the Ratzilla decision.
- **Phase 3 — 100% ODF 1.3, Imports & I/O (`L3` + `L6`):** `classic123-ods` reader/writer, `classic123-xlsx` and `classic123-csv` importers, `xtask ods-conformance` corpus suite, web remote `fetch()` loader and download, native filesystem open/save and HTTP loader, and opt-in `PUT`/`POST` save hooks on both targets.
- **Phase 4 — Polish, CLI & Parity Expansion:** `classic123-cli`, expanded OpenFormula library (lookup, financial, date, logical, string functions), cell number formatting/styles, column resizing by mouse drag, and WASM bundle size optimization.