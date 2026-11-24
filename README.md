# Classic123 (`rusted-classic-123`)

Ultra-lightweight, classic terminal-style (TUI) spreadsheet engineered in **Rust + WebAssembly (WASM)**. Designed for sub-millisecond responsiveness on a single HTML5 `<canvas>` with zero per-cell DOM overhead, hybrid **keyboard-first + full mouse** ergonomics, local & Docker execution, and **100% OASIS ODF 1.3 (`.ods`)** compatibility.

---

## Why Classic123?

Modern web spreadsheets suffer from heavy DOM trees, sluggish redraws, and multi-megabyte JavaScript bundles. **Classic123** moves the entire engine—sparse grid storage, OpenFormula parser, incremental DAG recalculation, `.ods` ZIP/XML codecs, TUI layout, and pixel-accurate mouse hit-testing—into pure Rust compiled to WebAssembly.

- **Zero DOM per Cell:** Renders a crisp monospace character buffer onto a single `<canvas>` using dirty-rectangle painting (< 16ms frame target, 60–120 FPS).
- **Keyboard Speed + First-Class Mouse:** Instant cell entry, hierarchical Slash Command Bar (`/`), and `POINT` mode alongside single-click select, double-click edit, click-and-drag range selection, scroll wheel, column resizing, and hover/click menus.
- **100% ODF 1.3 (`.ods`) Native:** Generates and reads compliant OpenDocument Spreadsheets (`mimetype` stored uncompressed first, valid XML manifest/content/styles/meta, canonical OpenFormula `of:=...`) that open cleanly in LibreOffice Calc, Collabora, and other ODF suites.
- **100% Client-Side & Easy to Self-Host:** Runs as pure static files on a personal website, locally via `trunk`, or inside a lightweight multi-stage Docker container. Load `.ods`/`.csv` files directly from your server via URL (`?file=/sheets/budget.ods`) and save locally or via optional `PUT`/`POST` hooks.
- **Headless CLI Included:** Inspect, recalculate, validate `.ods` conformance, and run scripted command batches without a browser via `classic123-cli`.

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

### 2. Run via Docker

Build and serve the optimized static WASM bundle with proper `application/wasm` and `.ods` MIME headers:

```sh
docker build -t classic123:local .
docker run --rm -p 8080:80 classic123:local
```

Mount a local folder of `.ods` spreadsheets to open them directly in the browser:

```sh
docker run --rm -p 8080:80 \
  -v "$PWD/corpus/valid:/usr/share/nginx/html/sheets:ro" \
  classic123:local
# Open http://127.0.0.1:8080/?file=/sheets/basic_formulas.ods
```

### 3. Run Headless CLI

```sh
cargo run --release -p classic123-cli -- inspect corpus/valid/basic_formulas.ods
cargo run --release -p classic123-cli -- recalc input.ods -o output.ods
```

---

## Architecture at a Glance

The repository is structured as a strictly unidirectional layered Rust workspace (`L0`–`L6`), enforced by `cargo xtask layers` and `unsafe_code = "forbid"`:

| Layer | Crates / Apps | Responsibility |
|---|---|---|
| **L0** | `classic123-coord`, `classic123-value`, `classic123-theme` | Cell addresses, typed values (`chrono` dates), high-contrast TUI palettes & glyph metrics |
| **L1** | `classic123-model`, `classic123-formula` | Sparse workbook/sheet grid, OpenFormula lexer/parser/AST/evaluator |
| **L2** | `classic123-calc`, `classic123-ops` | Incremental DAG recalculation, cycle detection (`#CIRC!`), reversible undo/redo deltas |
| **L3** | `classic123-ods`, `classic123-csv` | Pure-Rust OASIS ODF 1.3 (`.ods`) and RFC 4180 CSV codecs |
| **L4** | `classic123-engine` | Mode state machine (`READY`/`EDIT`/`MENU`/`POINT`), `CommandId` registry, `/` menu catalog |
| **L5** | `classic123-tui` | Character `CellBuffer`, dirty-rect diffing, unified `LayoutMap` for mouse hit-testing |
| **L6** | `apps/classic123-wasm`, `apps/classic123-cli`, `web/` | Single-`<canvas>` WASM presenter, minimal JS/TS static host, headless CLI |

---

## Documentation Index

- **Governance & AI Agent Rules:**
  - [`.specify/memory/constitution.md`](.specify/memory/constitution.md) — GitHub Spec Kit project constitution (non-negotiable principles).
  - [`AGENTS.md`](AGENTS.md) — Operational, clean-room, testing, and PR rules for human and AI contributors.
- **Technical & UX Design:**
  - [`docs/architecture.md`](docs/architecture.md) — Layered workspace design, sparse model, DAG engine, and thin-JS contract.
  - [`docs/interaction-spec.md`](docs/interaction-spec.md) — Keyboard modes, Slash Command Bar (`/`), and full mouse UX specification.
  - [`docs/odf-conformance.md`](docs/odf-conformance.md) — OASIS ODF 1.3 ZIP/XML and OpenFormula (`of:=...`) requirements.
  - [`docs/parity-checklist.md`](docs/parity-checklist.md) — Live command coverage matrix across Keyboard, Mouse, `/` Menu, and CLI.
  - [`docs/feature-names.md`](docs/feature-names.md) — Approved trademark-safe terminology and mode badges.
- **Development, Self-Hosting & Compliance:**
  - [`docs/development.md`](docs/development.md) — Toolchain setup, fast test loops, `cargo xtask` checks, and Docker workflow.
  - [`docs/web-integration.md`](docs/web-integration.md) — Embedding Classic123 on a personal site, remote `.ods` loading, server MIME/CORS config, and save hooks.
  - [`docs/contributing.md`](docs/contributing.md) — End-to-end recipes for adding commands/formulas and PR checklists.
  - [`SECURITY.md`](SECURITY.md) — Threat model (untrusted `.ods`/XML, CLI sandboxing) and vulnerability disclosure.
  - [`ATTRIBUTION.md`](ATTRIBUTION.md) — Clean-room provenance and licenses for bundled fonts and test corpus fixtures.

---

## Clean-Room Policy & License

**Classic123** is developed strictly as a clean-room implementation against public open standards (**OASIS ODF 1.3 / ISO/IEC 26300** and **RFC 4180**). See [`AGENTS.md`](AGENTS.md) and [`ATTRIBUTION.md`](ATTRIBUTION.md) for license and provenance requirements.