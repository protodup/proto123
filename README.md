# Proto123

Ultra-lightweight, classic terminal-style (TUI) spreadsheet engineered in **Rust**. One shared core runs in the browser via **WebAssembly (WASM)** on a single HTML5 `<canvas>` with zero per-cell DOM overhead, and locally as a **native terminal application**. Hybrid **keyboard-first + full mouse** ergonomics on both, under-16 ms input-to-frame latency, Docker self-hosting, and **100% OASIS ODF 1.3 (`.ods`)** output.

---

## Why Proto123?

Modern web spreadsheets suffer from heavy DOM trees, sluggish redraws, and multi-megabyte JavaScript bundles. **Proto123** moves the entire engine—sparse grid storage, OpenFormula parser, incremental DAG recalculation, `.ods` ZIP/XML codecs, Ratatui TUI layout, and exact mouse hit-testing—into one pure-Rust core that compiles to WebAssembly and to native binaries.

- **Zero DOM per Cell:** Renders a crisp monospace character buffer onto a single `<canvas>` (Ratzilla backend) with diffed redraws (< 16 ms input-to-frame, 60+ FPS).
- **Native Terminal App:** The same TUI runs in your terminal on Linux, macOS, and Windows (crossterm backend), opening and saving files on the local filesystem.
- **Keyboard Speed + First-Class Mouse (web and terminal):** Instant cell entry, hierarchical Slash Command Bar (`/`), and `POINT` mode alongside single-click select, double-click edit, click-and-drag range selection, mouse wheel, column resizing, and hover/click menus and dialogs.
- **100% ODF 1.3 (`.ods`) Native:** Generates and reads compliant OpenDocument Spreadsheets (`mimetype` stored uncompressed first, valid XML manifest/content/styles/meta, canonical OpenFormula `of:=...`) that open cleanly in LibreOffice Calc, Collabora, and other ODF suites. `.xlsx` and `.csv` files can be imported; output is always `.ods`.
- **100% Client-Side & Easy to Self-Host:** The web build runs as pure static files on a personal website, locally via `trunk` (including offline), or inside a lightweight multi-stage Docker container. Load `.ods`/`.xlsx`/`.csv` files directly from your server via URL (`?file=/sheets/budget.ods`) and save locally or via optional `PUT`/`POST` hooks.
- **Headless CLI Included:** Inspect, recalculate, validate `.ods` conformance, and run scripted command batches without a browser via `proto123-cli`.

---

## Quick Start

### 1. Run Locally (Web TUI on Canvas)

Prerequisites: Rust (`rustup` automatically selects the toolchain and `wasm32-unknown-unknown` target via `rust-toolchain.toml`) and `trunk`.

```sh
cargo install --locked trunk
trunk serve --open
```

Load a spreadsheet directly from a URL:

```text
http://127.0.0.1:8080/?file=/samples/budget.ods
```

### 2. Run in Your Terminal (Native)

```sh
cargo run --release -p proto123-term -- budget.ods
cargo run --release -p proto123-term -- https://example.com/sheets/budget.ods
```

Mouse support works in terminals with mouse reporting (most modern terminals); everything is also reachable by keyboard.

### 3. Run via Docker

Build and serve the optimized static WASM bundle with proper `application/wasm` and `.ods` MIME headers:

```sh
docker build -t proto123:local .
docker run --rm -p 8080:80 proto123:local
```

Mount a local folder of `.ods` spreadsheets to open them directly in the browser:

```sh
docker run --rm -p 8080:80 \
  -v "$PWD/corpus/valid:/usr/share/nginx/html/sheets:ro" \
  proto123:local
# Open http://127.0.0.1:8080/?file=/sheets/basic_formulas.ods
```

### 4. Run Headless CLI

```sh
cargo run --release -p proto123-cli -- inspect corpus/valid/basic_formulas.ods
cargo run --release -p proto123-cli -- recalc input.ods -o output.ods
cargo run --release -p proto123-cli -- convert input.xlsx -o output.ods
```

---

## Architecture at a Glance

The repository is structured as a strictly unidirectional layered Rust workspace (`L0`–`L6`), enforced by `cargo xtask layers` and `unsafe_code = "forbid"`:

| Layer | Crates / Apps | Responsibility |
|---|---|---|
| **L0** | `proto123-coord`, `proto123-value`, `proto123-theme` | Cell addresses, typed values (`chrono` dates), high-contrast TUI palettes & glyph metrics |
| **L1** | `proto123-model`, `proto123-formula` | Sparse workbook/sheet grid, OpenFormula lexer/parser/AST/evaluator |
| **L2** | `proto123-calc`, `proto123-ops` | Incremental DAG recalculation, cycle detection (`#CIRC!`), reversible undo/redo deltas |
| **L3** | `proto123-ods`, `proto123-xlsx`, `proto123-csv` | Pure-Rust OASIS ODF 1.3 (`.ods`) reader/writer; `.xlsx` and RFC 4180 CSV importers |
| **L4** | `proto123-engine` | Mode state machine (`READY`/`EDIT`/`MENU`/`POINT`), `CommandId` registry, `/` menu catalog |
| **L5** | `proto123-tui` | Shared Ratatui layout, `LayoutMap` and `InputMapper` for keyboard + mouse on both targets |
| **L6** | `apps/proto123-wasm`, `apps/proto123-term`, `apps/proto123-cli`, `web/` | Ratzilla `<canvas>` web target, crossterm native terminal target, minimal JS/TS static host, headless CLI |

---

## Documentation Index

- **Governance & AI Agent Rules:**
  - [`.specify/memory/constitution.md`](.specify/memory/constitution.md) — GitHub Spec Kit project constitution (non-negotiable principles).
  - [`AGENTS.md`](AGENTS.md) — Operational, clean-room, testing, and PR rules for human and AI contributors.
- **Technical & UX Design:**
  - [`docs/architecture.md`](docs/architecture.md) — Layered workspace design, sparse model, DAG engine, and thin-JS contract.
  - [`docs/interaction-spec.md`](docs/interaction-spec.md) — Keyboard modes, Slash Command Bar (`/`), and full mouse UX specification for web and terminal.
  - [`docs/odf-conformance.md`](docs/odf-conformance.md) — OASIS ODF 1.3 ZIP/XML and OpenFormula (`of:=...`) requirements.
  - [`docs/parity-checklist.md`](docs/parity-checklist.md) — Live command coverage matrix across Keyboard, Mouse, `/` Menu, and CLI.
  - [`docs/feature-names.md`](docs/feature-names.md) — Approved trademark-safe terminology and mode badges.
- **Development, Self-Hosting & Compliance:**
  - [`docs/development.md`](docs/development.md) — Toolchain setup, native terminal build, fast test loops, `cargo xtask` checks, and Docker workflow.
  - [`docs/web-integration.md`](docs/web-integration.md) — Embedding Proto123 on a personal site, remote `.ods` loading, server MIME/CORS config, and save hooks.
  - [`docs/contributing.md`](docs/contributing.md) — End-to-end recipes for adding commands/formulas and PR checklists.
  - [`SECURITY.md`](SECURITY.md) — Threat model (untrusted `.ods`/`.xlsx`/XML, native file and network I/O, CLI sandboxing) and vulnerability disclosure.
  - [`ATTRIBUTION.md`](ATTRIBUTION.md) — Clean-room provenance and licenses for bundled fonts and test corpus fixtures.

---

## Clean-Room Policy & License

**Proto123** is developed strictly as a clean-room implementation against public open standards (**OASIS ODF 1.3 / ISO/IEC 26300** and **RFC 4180**). See [`AGENTS.md`](AGENTS.md) and [`ATTRIBUTION.md`](ATTRIBUTION.md) for license and provenance requirements.