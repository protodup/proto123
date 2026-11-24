# rusted-classic-123 Constitution

**Vision and Identity**: rusted-classic-123 is a terminal-style (TUI) spreadsheet application
inspired by the speed and ergonomics of Lotus 1-2-3, engineered to run 100% client-side in the
browser via Rust + WebAssembly (WASM) on a static site. Its mission is to eliminate the bloat, lag,
and heavy abstractions of modern web spreadsheets — delivering near-instant responsiveness, modern
usability, and strict adherence to open document standards.

## Core Principles

### I. Extreme Lightweight Performance & Zero DOM Overhead

- **No DOM per cell**: The spreadsheet grid and TUI interface MUST NOT be rendered as HTML DOM
  elements (`<div>`, `<table>`, `<td>`, or equivalents). All visual rendering MUST occur inside a
  single `<canvas>` (2D or WebGL) surface using a monospace font.
- **Optimized redraw**: Rendering MUST use a character-cell buffer and/or dirty-rect strategy so
  that only changed regions are redrawn; full-surface redraws are reserved for resize and theme
  changes.
- **Performance budget**: End-to-end latency from any input event (keypress, mouse action, wheel)
  to the corresponding visual frame update MUST stay under 16 ms (60+ FPS) on the reference
  workloads defined in each feature plan.
- **Binary size discipline**: The release `.wasm` bundle MUST be optimized with `wasm-opt` and
  served compressed. Its compressed size MUST be measured on every release build, and any new
  dependency or size increase MUST be justified in the feature plan.
  TODO(WASM_SIZE_BUDGET): adopt a numeric ceiling once a baseline exists.

**Rationale**: Per-cell DOM is the root cause of lag in web spreadsheets; a single canvas with a
cell buffer keeps cost proportional to what changed, not to sheet size.

### II. Rust + WebAssembly Core

- All business logic, spreadsheet state, formula evaluation, dependency graph (DAG), file
  serialization/deserialization, and TUI layout and hit-testing MUST reside in Rust compiled to
  WASM.
- The JavaScript/TypeScript layer MUST remain a minimal thin shell, limited to:
  1. Initializing the WASM module.
  2. Capturing raw keyboard, mouse, wheel, and window-resize events and forwarding them to WASM
     without interpreting them.
  3. Performing browser-level I/O (fetch, file download, optional HTTP upload) when requested by
     the Rust engine.
- JavaScript MUST NOT contain spreadsheet rules, formula logic, layout decisions, or file-format
  knowledge.

**Rationale**: One language owning all behavior keeps the logic testable with `cargo test` and
prevents divergence between two implementations of the same rules.

### III. Hybrid Navigation: Keyboard-First with Full Mouse Support

- **Keyboard completeness**: Every feature MUST be fully operable from the keyboard: arrow/grid
  navigation, direct shortcuts, immediate cell data entry, and the classic hierarchical `/`
  command menu.
- **Mouse in the grid**: Single-click MUST select a cell, double-click MUST enter edit mode,
  click-and-drag MUST select a range, and the mouse wheel MUST scroll vertically (and
  horizontally with the platform's horizontal-scroll gesture or modifier).
- **Mouse in menus and dialogs**: Every top-bar menu item (e.g., File, Range, Edit) and every
  dialog control MUST show a hover highlight and respond to direct clicks.
- **Exact hit-testing**: Canvas pixel coordinates `(x, y)` MUST map deterministically to TUI cell
  coordinates `(col, row)`, accounting for device pixel ratio, font metrics, and scroll offset.
  This mapping MUST live in Rust and be unit-tested.

**Rationale**: The classic keyboard flow is the product's identity; mouse parity removes the
learning barrier for users who never used a TUI.

### IV. 100% ODS (OpenDocument / ODF) Compatibility

- **Native format**: Every file generated or exported MUST be a valid ODS document compliant with
  OASIS ODF 1.3 (ISO/IEC 26300).
- **Package integrity**: Every `.ods` archive produced in memory MUST follow the ODF ZIP container
  rules:
  - `mimetype` MUST be the first ZIP entry, stored uncompressed (STORED), with the exact content
    `application/vnd.oasis.opendocument.spreadsheet` and no trailing newline.
  - `content.xml`, `styles.xml`, `meta.xml`, and `META-INF/manifest.xml` MUST be present,
    well-formed, and consistent with the manifest.
- **OpenFormula**: All formulas written to `.ods` MUST use OpenFormula syntax (`of:=...`) so they
  recalculate faithfully and open without warnings in ODF-compliant suites (LibreOffice Calc,
  Collabora, Microsoft Excel).
- **Fidelity**: Export MUST preserve cell value types (number, string, boolean, date, percentage,
  currency), multiple sheets, and formulas. Import of `.xlsx` and `.csv` is supported as input;
  output is always ODS.

**Rationale**: An open, standardized native format guarantees users own their data and can move
it to any compliant suite without loss.

### V. Backend Independence, Multi-Environment Execution & Flexible I/O

- **Pure static delivery**: The application MUST deploy and run as static assets (HTML, JS, WASM)
  with no backend required for calculation, rendering, or file generation.
- **Local execution**: The project MUST run locally for development and offline/localhost use with
  minimal tooling (Rust toolchain plus the chosen WASM bundler).
- **Docker execution**: The project MUST ship a lightweight, reproducible multi-stage `Dockerfile`
  that builds the assets and serves them with the correct `application/wasm` MIME type.
- **Remote loading**: The application MUST load `.ods`, `.xlsx`, or `.csv` files from a
  user-supplied URL via `fetch` → `Uint8Array` → WASM linear memory. Failures (network, CORS,
  unsupported or corrupt file) MUST be reported to the user without losing current work.
- **Persistence**: The application MUST allow downloading the current workbook as `.ods`, and MUST
  expose an opt-in hook to send the updated binary to a configured endpoint via HTTP `PUT` or
  `POST`. No upload occurs unless explicitly configured.

**Rationale**: Static hosting is cheap, private, and durable; data leaves the browser only when the
user decides.

### VI. Efficient Data & Calculation Engine

- Workbook storage MUST be sparse — either a sparse map (e.g., `HashMap<(u32, u32), Cell>`) or a
  proven Rust calculation core (e.g., `ironcalc_base`) paired with an ODS serialization layer.
- Recalculation MUST be driven by a dependency graph (DAG): an edit recomputes only the edited
  cell and its transitive dependents, in topological order.
- Circular references MUST be detected and surfaced as a cell error, never cause unbounded
  recursion or a hang.

**Rationale**: Memory and recalculation cost must scale with the cells actually used and changed,
not with the theoretical grid size.

## Technical Constraints & Standard Stack

- **Language**: Rust, latest stable toolchain and edition.
- **Target**: `wasm32-unknown-unknown`, built with `wasm-pack` or `trunk`.
- **TUI rendering**: Ratatui + Ratzilla (Canvas/WebGL backend), OR a custom Rust/WASM Canvas 2D
  character-grid renderer with exact pixel-to-cell hit-testing. The choice MUST be recorded in the
  first feature plan that introduces rendering.
- **ODS serialization**: Pure Rust (e.g., `spreadsheet-ods`, or a custom `zip` + `quick-xml`
  reader/writer), and every chosen crate MUST compile for `wasm32-unknown-unknown`.
- **Prohibitions**: Heavy frontend frameworks (Angular, Next.js, React-scale UI kits) and bulky
  CSS/UI libraries MUST NOT be introduced. Any web wrapper MUST be minimal vanilla HTML/JS/TS.

## UI/UX Guidelines

- **Aesthetic**: Classic terminal presentation — high contrast, crisp monospace typography,
  ASCII/Unicode box-drawing borders, a clear active-cell indicator, a top formula/command bar,
  and a bottom status/mode bar.
- **Visual feedback**: Focus, range selection, menu hover, and the current mode (`READY`, `EDIT`,
  `MENU`, `POINT`, and any mode added later) MUST be visually unambiguous at all times.

## Development Workflow & Quality Gates

Every feature specification (`/speckit-specify`) and implementation plan (`/speckit-plan`) MUST
include a Constitution Check that explicitly verifies:

1. Rendering remains free of per-cell DOM elements (Principle I).
2. Every interaction added works via both keyboard and mouse (Principle III).
3. Core logic and file serialization stay in Rust, with no business rules in JavaScript
   (Principle II).
4. Generated `.ods` files remain ODF-compliant, preserving data types, multiple sheets, and
   OpenFormula syntax (Principle IV).
5. The 16 ms input-to-frame budget and the wasm size measurement are addressed (Principle I).

A feature is complete only when automated Rust tests (`cargo test`) pass and cover, as applicable
to the change: the formula engine, the dependency graph (including cycle detection),
pixel-to-TUI hit-testing, and `.ods` generation with round-trip integrity (write → read → compare).

## Governance

- This constitution supersedes all other project practices. Where a spec, plan, or task conflicts
  with it, the constitution wins until it is formally amended.
- **Amendments** are made via `/speckit-constitution`, MUST state the motivation, and MUST update
  the version and the Last Amended date. Amendments that relax a principle MUST describe the
  migration impact on existing features.
- **Versioning** follows semantic versioning:
  - MAJOR — removal or backward-incompatible redefinition of a principle or gate.
  - MINOR — new principle or section, or materially expanded guidance.
  - PATCH — clarifications and wording fixes with no semantic change.
- **Compliance review**: Every plan's Constitution Check and every code review MUST confirm the
  Quality Gates above. Any justified deviation MUST be recorded in the plan's Complexity Tracking
  with the reason and the simpler alternative that was rejected.

**Version**: 1.0.0 | **Ratified**: 2026-11-24 | **Last Amended**: 2026-11-24
