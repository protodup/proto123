# Implementation Plan: Foundational MVP — Calculating Grid on a Single Canvas

**Branch**: `001-foundational-mvp` | **Date**: 2026-11-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `.specify/specs/001-foundational-mvp/spec.md`

## Summary

Build the first vertical slice of Proto123: one in-memory sparse worksheet with typed values, label prefixes, an OpenFormula-precedence formula language (`+ - * / ^`, references, ranges, `SUM`/`AVERAGE`/`MIN`/`MAX`/`COUNT`), incremental dependency-graph recalculation with `#CIRC!` detection, reversible Range Erase/Copy/Move and Undo/Redo, a command-driven session (`READY`/`VALUE`/`LABEL`/`EDIT`/`POINT`/`MENU`/`DIALOG`), and a shared Ratatui layout drawn on the web by Ratzilla's Canvas backend (one `<canvas>`, one ARIA live region) and natively by crossterm (including the Linux-console profile). Mouse gestures, pixel→cell mapping, and `LayoutMap` hit-testing live in `proto123-tui`. Guardrails: `cargo xtask layers`, `cargo xtask menu-commands`, `cargo xtask wasm` (size report), a Trunk local-serve command, and a multi-stage Docker image.

Technical approach (see [research.md](research.md)): no file I/O, no CLI, no L3 crates in this slice. Ratzilla is used **only as a drawing backend**; all input is forwarded raw by the hosts (JS on the web, crossterm loop natively) as timestamped `RawEvent`s, because Ratzilla's own event API has no wheel events, no timestamps, and does its own pixel→cell conversion, which the constitution requires to live in `proto123-tui`.

## Technical Context

**Language/Version**: Rust stable 1.99.1, edition 2024, pinned in `rust-toolchain.toml` (components `clippy`, `rustfmt`; target `wasm32-unknown-unknown`).

**Primary Dependencies** (all MIT or MIT/Apache-2.0; see research R2–R4):
- Shared core L0–L4: **none** beyond `std` (no `chrono` — no dates in this slice).
- L5 `proto123-tui`: `ratatui` 0.30 (`default-features = false`), `unicode-width` 0.2.
- L6 web `apps/proto123-wasm`: `ratzilla` 0.3 (Canvas backend only), `wasm-bindgen` 0.2, `console_error_panic_hook` (via Ratzilla).
- L6 native `apps/proto123-term`: `ratatui` 0.30 (`crossterm` feature), `crossterm` 0.29.
- `xtask` (native-only tooling): `cargo_metadata`, `flate2`.
- Tooling (not crates): `trunk` 0.22 (bundles, runs `wasm-bindgen` and `wasm-opt`), Docker, `nginx:alpine` runtime image.

**Storage**: N/A. In-memory sparse `HashMap<CellAddr, Cell>`; no persistence (deferred to Spec 002).

**Testing**: `cargo test` (one `tests/it/main.rs` binary per crate), Ratatui `TestBackend` for frame/console-profile assertions, `proto123-testkit` shared scenario suite (US1–US5) replayed through each target's input translation (FR-049), ignored `panic_hunt` hostile-input sweep in `proto123-engine`, `cargo clippy -D warnings`.

**Target Platform**: `wasm32-unknown-unknown` in evergreen desktop browsers; native Linux (incl. virtual console `TERM=linux`, SSH, tmux), macOS, Windows terminals. No X11/Wayland/GPU.

**Project Type**: Cargo workspace — layered libraries + two front-end apps (web canvas, native terminal) + `xtask`.

**Performance Goals**: ≤ 16 ms input→frame on a 10,000-populated-cell sheet (5,000 formulas) on both targets (SC-002); an edit touching 10 dependents evaluates exactly those 10 (SC-003); 10,000-deep chains and 10,000-long cycles without recursion.

**Constraints**: zero per-cell DOM; one `<canvas>` + one live region; no panics in non-test code; `unsafe_code = "forbid"` everywhere; L0–L5 compile for wasm32 and native; release `.wasm` built with `wasm-opt -Oz` and its gzip size reported; 80×24 minimum; all UI strings from one catalog.

**Scale/Scope**: 1 sheet, A1:XFD1048576 address space; 7 modes; 10 menu leaves/branches (`File→New`, `Edit→Undo,Redo`, `Range→Erase,Copy,Move`, `View→Theme`); 3 built-in themes × 2 glyph sets; ~35 command IDs.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design (below).*

| # | Gate (constitution v1.2.0) | Status | Evidence in this plan |
|---|---|---|---|
| 1 | Web free of per-cell DOM (I) | ✅ PASS | Ratzilla `CanvasBackend` paints one `<canvas>`; `web/index.html` has exactly `<canvas>` + one `aria-live` div ([contracts/wasm-bridge.md](contracts/wasm-bridge.md)). Ratzilla's DOM backend is not used. |
| 2 | Keyboard **and** mouse for every interaction (III) | ✅ PASS | Every command in [contracts/commands.md](contracts/commands.md) lists its key and mouse path; menu leaves reachable by letter, arrows, hover+click. |
| 3 | Logic in shared core, none in JS/native shell (II) | ✅ PASS | Hosts only forward `RawEvent`s ([contracts/raw-input.md](contracts/raw-input.md)); gestures (`InputMapper`), pixel→cell, `LayoutMap`, capability-profile rules all in `proto123-tui`/`proto123-theme`. |
| 4 | Works on both targets / differences documented (II) | ✅ PASS | Same core + layout; only FR-050 differences (live region web-only, pixel step web-only, native mouse depends on terminal). Shared scenario suite runs against both translations. |
| 5 | ODF compliance (IV) | ✅ N/A (deferred) | No file output in this slice. AST + canonical text follow OpenFormula precedence/function names so Spec 002 can emit `of:=` without changing semantics. |
| 6 | 16 ms budget + wasm size measured (I) | ✅ PASS | Event-driven redraw with Ratatui + Ratzilla cell diffing; `frame_budget` native measurement and browser `performance.now()` check in quickstart; `cargo xtask wasm` reports raw + gzip size. |
| 7 | Minimal-terminal profile (V) | ✅ PASS | `console16` theme (ANSI-16 only, no bold/underline), single-line + ASCII glyph sets, `F8` extend mode, every `Ctrl+` command also in `/` menu, no-mouse completeness; `TERM=linux` detection in `proto123-theme`. |

Additional AGENTS.md rules checked: no new dependency outside approved licences (all MIT/Apache-2.0); every crate has `[lints] workspace = true`; no `#[allow]` of lints; string catalog (rule 17); trademark-safe labels (rule 23); `.github/workflows/` and `Dockerfile` are explicitly in this spec's scope (US7), so rule 18 does not block them.

**Result: PASS — no unjustified violations.** One accepted risk is tracked in Complexity Tracking.

### Post-design re-check (after Phase 1)

Re-evaluated against [data-model.md](data-model.md) and [contracts/](contracts/): no gate changes. The design adds no browser/terminal types below L5, keeps the formula AST target-neutral, and records the renderer decision (Ratzilla Canvas backend, drawing only) as the constitution requires for the first rendering plan. **PASS.**

## Project Structure

### Documentation (this feature)

```text
.specify/specs/001-foundational-mvp/
├── plan.md              # This file
├── research.md          # Phase 0: decisions R1–R14
├── data-model.md        # Phase 1: entities, invariants, state machines
├── quickstart.md        # Phase 1: runnable validation guide
├── contracts/
│   ├── commands.md      # CommandId registry + menu tree + key/mouse bindings
│   ├── raw-input.md     # RawEvent contract shared by both hosts
│   ├── wasm-bridge.md   # JS host ↔ WASM exports, index.html shape
│   ├── native-cli.md    # proto123-term arguments, profile override, exit behavior
│   ├── formula-grammar.md # Entry classification + formula grammar + display format
│   └── xtask.md         # cargo xtask subcommands, outputs, exit codes
├── checklists/requirements.md
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
Cargo.toml                    # [workspace], [workspace.lints] unsafe_code="forbid", clippy unwrap/expect/panic/indexing lints, profiles incl. release-wasm
rust-toolchain.toml           # 1.99.1 + wasm32-unknown-unknown, clippy, rustfmt
Trunk.toml                    # target = web/index.html, dist = dist/
Dockerfile                    # builder (rust + trunk) → nginx:alpine runtime
docker/nginx.conf             # application/wasm, gzip
.cargo/config.toml            # alias xtask = "run -p xtask --"
crates/
├── proto123-coord/     L0    # CellAddr, CellRange, A1 parse/format with $ flags, grid limits
├── proto123-value/     L0    # Value, ErrorKind, general number display (FR-006)
├── proto123-theme/     L0    # Theme tokens, Rgb/Ansi16 colors, GlyphSet, contrast math, CapabilityProfile::detect
├── proto123-formula/   L1    # lexer, Pratt parser, Expr AST, canonical printer, ref shift/remap, evaluator over CellSource
├── proto123-model/     L1    # Cell, CellContent, Alignment, Sheet (sparse), Workbook (1 sheet)
├── proto123-calc/      L2    # DependencyGraph, CalcSheet (sheet + graph), dirty-subgraph Kahn recalc, #CIRC!
├── proto123-ops/       L2    # CellDelta, Erase/Copy(tile)/Move ops, UndoStack
├── proto123-engine/    L4    # Session, Mode, CommandId registry, dispatch, menu_catalog.rs, strings.rs, entry classification
├── proto123-tui/       L5    # Ratatui render(), LayoutMap, HitTarget, CellMetrics + pixel_to_cell, InputMapper, theme→ratatui mapping
└── proto123-testkit/   support (dev-dependency only)  # scenario DSL + US1–US5 scenarios, hostile-input generators
apps/
├── proto123-wasm/      L6    # wasm-bindgen exports + Ratzilla CanvasBackend; translate.rs is target-neutral (tested natively)
└── proto123-term/      L6    # crossterm loop, TerminalGuard, env→profile, translate.rs
web/
├── index.html                # <canvas>, one aria-live div, trunk links
├── host.js                   # ~150 lines vanilla JS: boot, forward raw events, write live region
└── style.css                 # fullscreen canvas, focus ring, .sr-only
xtask/src/{main.rs,layers.rs,menu.rs,wasm.rs,docker.rs}
.github/workflows/ci.yml      # fmt, clippy, test, layers, menu-commands, wasm, term build (linux/macos/windows), docker-check
docs/                         # update interaction-spec.md §3/§5, feature-names.md §4, parity-checklist.md, architecture.md "Current state"
```

Each crate uses `src/lib.rs` + `tests/it/main.rs` (AGENTS rule 31). Not created in this slice: `proto123-ods`, `proto123-xlsx`, `proto123-csv`, `apps/proto123-cli`, `corpus/` ODS fixtures (Spec 002). `xtask/src/layers.rs` still lists them so adding them later only flips them from "absent" to checked.

**Structure Decision**: The L0–L6 workspace from `AGENTS.md` / `docs/architecture.md`, instantiated only for the crates this slice exercises. Two documented refinements: (a) **same-layer edges are allowed when acyclic** (`proto123-model → proto123-formula` in L1, `proto123-ops → proto123-calc` in L2) — `cargo` already rejects cycles and the layer check rejects only upward edges; (b) `proto123-testkit` is a support crate that may depend on L0–L5 and may only be used as a `[dev-dependencies]` entry, which `xtask layers` enforces.

## Implementation phases (input to /speckit-tasks)

1. **Workspace & guardrails** — root `Cargo.toml`, toolchain, lint config, `xtask layers` + `menu-commands` (with seeded-violation tests), CI skeleton.
2. **Core L0–L2** — coord → value (display format) → formula (parser, printer, evaluator, shifting) → model → calc (graph, recalc, cycles) → ops (erase/copy/move, undo). Each with exact-value tests from US2 and Edge Cases.
3. **Engine L4** — session/mode machine, command registry + dispatch, menu catalog, string catalog, prompts (`DIALOG`), entry classification + diagnostics.
4. **TUI L5** — layout/render, `LayoutMap`, `CellMetrics`/`pixel_to_cell`, `InputMapper` (keys, click, double-click 500 ms, drag + auto-scroll, wheel 3 rows, hover), themes→ratatui, console-profile frame assertions, live-region text.
5. **Targets L6** — `proto123-term` (guard, mouse capture, profile detection, resize, < 80×24 message); `proto123-wasm` + `web/` (Ratzilla canvas, raw forwarding, live region); shared scenario suite on both translations.
6. **Delivery** — Trunk serve, `xtask wasm` size report, Dockerfile + `xtask docker-check`, docs updates (interaction-spec §3/§5, feature-names §4, parity checklist, architecture current state), `ATTRIBUTION.md` unchanged (no fonts/fixtures added).

## Complexity Tracking

| Item | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| Ratzilla 0.3 hard-depends on `beamterm-renderer` (WebGL2, MIT) even though only the Canvas backend is used | Constitution names Ratzilla as the default web backend; it already does per-cell dirty redraw (FR-033) | A custom Canvas 2D painter is only allowed if Ratzilla misses the 16 ms budget; nothing suggests it will. Unused WebGL code is expected to be removed by LTO + `wasm-opt`; `cargo xtask wasm` measures it, and the first size report becomes the baseline for `TODO(WASM_SIZE_BUDGET)`. |
| Risk: `#[wasm_bindgen]` expansion under `unsafe_code = "forbid"` on the wasm32 target (verified clean on the native target only) | Workspace-wide `forbid` is mandated by AGENTS.md | If the wasm32 expansion trips the lint, the fallback is a narrow, listed exception for `apps/proto123-wasm/src/bridge.rs` only, added to AGENTS.md "This repository" with `// SAFETY:` notes — this needs operator approval and is not taken pre-emptively. |
