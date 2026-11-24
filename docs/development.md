# Development Guide

This guide covers setting up a local environment, building the WebAssembly app and headless CLI, running fast test loops, validating ODF 1.3 (`.ods`) conformance, and running the multi-stage Docker container for **Classic123** (`rusted-classic-123`).

---

## Prerequisites

- **Rust toolchain:** `rust-toolchain.toml` pins the exact Rust release used by CI, and `rustup` installs it together with the `wasm32-unknown-unknown` target automatically on first use. (`rust-version` in root `Cargo.toml` records the oldest supported MSRV.)
- **WebAssembly tooling:**
  - [`trunk`](https://trunkrs.dev/) (`cargo install --locked trunk`) for local WASM dev serving and static bundling (or `wasm-pack` if building the raw JS/WASM package directly).
  - `wasm-opt` (from [Binaryen](https://github.com/WebAssembly/binaryen)) to enforce the `.wasm` release size budget.
- **Optional recommended tools:**
  - [`cargo-nextest`](https://nexte.st/) (`cargo install --locked cargo-nextest`) for parallel test execution.
  - **Docker** (Engine or Desktop) for testing the production static container and MIME-type configuration.
  - **LibreOffice (`soffice`)** *(optional)* for cross-checking generated `.ods` files against an external ODF 1.3 implementation during local verification.
- **No native GUI/GPU libraries required:** Unlike desktop GUI apps, `rusted-classic-123` requires **no** GTK, Wayland, X11, OpenGL, or Vulkan system packages on Linux, macOS, or Windows. Everything runs in pure Rust (CLI/tests) or inside the browser's `<canvas>` via WASM.

### Windows Source Builds

Use the Rust MSVC toolchain and install Visual Studio or Microsoft C++ Build Tools with the **Desktop development with C++** workload. Open a fresh terminal and verify `cargo --version`, `rustc --version`, and `rustup target list --installed` (confirming `wasm32-unknown-unknown` is present).

On Windows 11, **Smart App Control** can occasionally block `cargo`, `rustc`, or freshly compiled test binaries with `An Application Control policy has blocked this file. (os error 4551)`. Moving `CARGO_TARGET_DIR` or reinstalling Rust will not bypass an OS application-control policy; check **Windows Security -> App & browser control -> Smart App Control settings** (or Event Viewer under `Microsoft -> Windows -> CodeIntegrity -> Operational` on managed machines), or develop inside WSL2 / Docker.

---

## Build and Run

### 1. Local Web App (Browser TUI on Canvas)

Serve the WASM app locally with hot-reload:

```sh
trunk serve --open
```

To test loading a spreadsheet directly from a local/remote URL via `fetch()`, place a sample `.ods` file in `web/samples/` (or any static endpoint) and pass the query parameter:

```text
http://127.0.0.1:8080/?file=/samples/budget.ods
```

To build the optimized production static bundle (`dist/`):

```sh
trunk build --release
cargo xtask wasm          # verifies wasm32 build, runs wasm-opt -Oz, and checks binary size budget
```

### 2. Headless CLI (`classic123-cli`)

Every engine operation is accessible without a browser:

```sh
# Inspect structure, sheets, formula counts, and ODF 1.3 diagnostics
cargo run --release -p classic123-cli -- inspect corpus/valid/basic_formulas.ods

# Recalculate DAG and write a fresh ODF 1.3 compliant .ods
cargo run --release -p classic123-cli -- recalc input.ods -o output.ods

# Execute scripted commands in sandboxed automation mode (enforces root boundaries)
cargo run --release -p classic123-cli -- exec script.json \
  --read-root ./work/in \
  --write-root ./work/out
```

### 3. Docker Execution (Local & Production Parity)

Build and run the multi-stage container (compiles Rust -> WASM + `wasm-opt` in the builder stage, serves static assets from a minimal server in the runtime stage):

```sh
docker build -t classic123:local .
docker run --rm -p 8080:80 classic123:local
```

To mount a local directory of `.ods` spreadsheets so the containerized app can load them via `fetch()`:

```sh
docker run --rm -p 8080:80 \
  -v "$PWD/corpus/valid:/usr/share/nginx/html/sheets:ro" \
  classic123:local
# Then open: http://127.0.0.1:8080/?file=/sheets/basic_formulas.ods
```

Verify the container build and required HTTP headers (`Content-Type: application/wasm` for `.wasm` and `Content-Type: application/vnd.oasis.opendocument.spreadsheet` for `.ods`):

```sh
cargo xtask docker-check
```

---

## Workspace Gate & `xtask` Commands

Run the full verification gate or targeted checks via `cargo xtask`:

```sh
cargo test --workspace            # run all unit, integration, and doc tests
cargo xtask test                  # runs cargo-nextest (when installed) + doctests
cargo xtask ci                    # full CI gate: fmt, clippy, layers, menu-commands, tests, panic_hunt, doctests, wasm, ods-conformance
cargo xtask ci lint               # fast lint leg (fmt, clippy, layers, menu-commands) with no test binaries
cargo xtask layers                # verify L0-L6 unidirectional dependency graph (xtask/src/layers.rs)
cargo xtask menu-commands         # verify every '/' menu entry resolves to a live CommandId
cargo xtask wasm                  # compile wasm32-unknown-unknown + enforce .wasm size budget
cargo xtask ods-conformance       # validate ODF 1.3 ZIP/XML/OpenFormula round-trips across corpus/
cargo xtask parity                # regenerate docs/parity-checklist.md (Keyboard / Mouse / CLI coverage)
cargo xtask docker-check          # verify multi-stage Dockerfile and WASM/ODS MIME types
cargo xtask stats                 # test counts and line counts per crate
```

### Build Profile Tuning

- **Fast dependencies in dev/test:** Parsing ZIP/XML (`classic123-ods`) and running large DAG recalculations can be sluggish at `opt-level = 0`, so the workspace `Cargo.toml` compiles third-party dependencies at `opt-level = 2` while keeping workspace crates at `opt-level = 0` for fast incremental rebuilds.
- **Faster linking (`line-tables-only`):** The `dev` profile builds `classic123-*` crates with `debug = "line-tables-only"` and dependencies with `debug = false`. Panic backtraces preserve exact file and line numbers in project code while cutting test-binary link time roughly in half. When you need full local variable inspection in `lldb`/`gdb`, override per invocation:

```sh
CARGO_PROFILE_DEV_DEBUG=true cargo test -p classic123-calc
```

- **WASM release profile:** `[profile.release-wasm]` enables `opt-level = "z"`, `lto = true`, `codegen-units = 1`, and `panic = "abort"`, followed by `wasm-opt -Oz` to keep cold-start downloads sub-second.

---

## Faster Test Loops

Most of a raw `cargo test` run is spent linking binaries and executing sequentially. While iterating on a single crate:

```sh
cargo test -p classic123-formula --lib --tests         # skip examples and doctests
cargo test -p classic123-ods --test it roundtrip::     # run one integration-test module (tests/it/roundtrip.rs)
cargo nextest run --workspace                          # run tests in parallel processes
cargo test --doc --workspace                           # nextest does not execute doctests
```

### Hostile-Input Sweeps & Long Matrix Tests

The engine, formula parser, and `.ods` reader include hostile-input sweeps (corrupt ZIP archives, zip-bomb decompression limits, malformed XML, deeply nested formulas, circular reference chains, extreme coordinates, and fuzz-like command sequences):

```sh
# Run the ignored panic-hunt command sweep
cargo test -p classic123-engine --test panic_hunt -- --ignored
# Or with nextest:
cargo nextest run -p classic123-engine --run-ignored only
```

To keep daily PR runs fast while still exercising the full matrix over time, `classic123-testkit` sweeps partition large matrices by UTC day unless overridden:

- `CLASSIC123_FULL_SWEEP=1` — runs **100%** of sweep cases (used by nightly CI and recommended as a final local check before merging parser/ODS changes).
- `CLASSIC123_SWEEP_PART=<n>` — replays the exact partition printed by a failing CI run.

Nextest's `SLOW` notice at 60 seconds is an informational progress marker, not a failure. Never silence a slow test by skipping cases or raising thresholds without documenting real algorithmic gains under identical build/environment conditions.

---

## Headless Testing of TUI Layout & Mouse Hit-Testing

Because `classic123-tui` renders to a pure-Rust `CellBuffer` and emits a deterministic `LayoutMap` from `CellMetrics`, you can verify **keyboard navigation, `/` menus, mouse clicks, double-clicks, drag-range selection, scroll wheel behavior, and hover highlights** without launching a browser:

```sh
cargo test -p classic123-tui --lib --tests
```

To visually inspect a rendered frame (e.g., checking box-drawing borders, active range highlighting, or menu hover states) without starting a browser, set `CLASSIC123_TUI_SNAPSHOTS` to a temporary directory when running snapshot-capable TUI tests:

```sh
(
  set -eu
  snap_dir="$(mktemp -d)"
  CLASSIC123_TUI_SNAPSHOTS="$snap_dir" cargo test -p classic123-tui --test it tui_snapshots::
  ls -la "$snap_dir"
)
```

Inspect the generated ASCII/ANSI or rendered frame dumps in `$snap_dir` and remove the temporary directory afterward. Never commit scratch snapshots to the repository.

---

## Fonts and WASM Binary Budget

Monospace fonts for the Canvas TUI are **never embedded inside the `.wasm` binary**. Embedding even a single Unicode monospace typeface bloats the `.wasm` payload by hundreds of kilobytes to several megabytes and defeats our zero-overhead cold-start budget.

Instead:
- `web/fonts/` (or static assets served beside `index.html`) hosts an optional permissively licensed monospace `.woff2` font (e.g., SIL OFL-1.1 / MIT / Apache-2.0) loaded asynchronously by the browser via CSS `@font-face` / `document.fonts`, with an immediate fallback to the OS `monospace` font so the Canvas renders on frame 1 with zero blocking.
- `classic123-tui` measures actual glyph `CellMetrics { char_width, char_height, dpr }` dynamically whenever the font loads or the window resizes, automatically recalculating both `CellBuffer` dimensions and mouse hit-test targets.
- Any bundled font file MUST have its source and license (`OFL.txt` / `LICENSE`) recorded in `ATTRIBUTION.md`.

---

## ODF 1.3 Conformance & Test Corpus (`corpus/`)

`corpus/` stores clean-room `.ods` fixtures and hostile-input archives:

- `corpus/valid/` — minimal, formula-heavy, multi-sheet, date/currency, and repeated-column/row `.ods` files created clean-room.
- `corpus/hostile/` — truncated ZIPs, wrong `mimetype` compression/order, cyclic references, extreme coordinates, and invalid UTF-8/XML inputs used by panic sweeps.

Run the conformance suite:

```sh
cargo xtask ods-conformance
```

If **LibreOffice** is installed locally, you can perform an extra external sanity check on any `.ods` file generated by `classic123-cli` or exported from the browser:

```sh
soffice --headless --convert-to csv --outdir /tmp output.ods
```

Every fixture added to `corpus/` MUST be original work or permissively licensed under clean-room rules and logged in `ATTRIBUTION.md` in the same PR.

---

## Spec Kit & PR Workflow Alignment

When implementing a feature or fix using **GitHub Spec Kit**:

1. Check `.specify/memory/constitution.md`, `AGENTS.md`, and the active spec/tasks under `.specify/specs/`.
2. Implement changes respecting `L0`–`L6` boundaries (`cargo xtask layers`) and zero-DOM / keyboard+mouse parity.
3. Add targeted behavioral/invariant tests (and regression inputs in `corpus/hostile/` for any crash/panic fix).
4. Run the relevant gate checks before opening a PR:

```sh
cargo xtask ci lint
cargo test -p <modified-crate>
cargo xtask wasm
cargo xtask ods-conformance
```

5. Report in the PR description exactly which checks ran and passed (never mark an unrun check as passed), and keep local scratch files (`log/devlog.md`, `plan/`, `.claude/`, `.cursor/`) uncommitted.