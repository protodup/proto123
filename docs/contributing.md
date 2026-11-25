# Contributing to Proto123

Thank you for contributing to **Proto123**. This guide provides practical end-to-end recipes for implementing features cleanly across our layered Rust architecture (web/WASM and native terminal targets) and **GitHub Spec Kit** workflow.

Before writing code, read:
1. [`.specify/memory/constitution.md`](../.specify/memory/constitution.md) — Non-negotiable architectural principles.
2. [`AGENTS.md`](../AGENTS.md) — Operational, clean-room, testing, and PR rules.
3. [`docs/architecture.md`](architecture.md) — L0–L6 crate boundaries and core data flow.

---

## 1. GitHub Spec Kit Workflow

New features follow the Spec Kit lifecycle so design decisions are verified before code is written:

1. **Specify (`/speckit-specify`):** Define user scenarios, keyboard + mouse behavior on both targets, and ODF 1.3 expectations under `.specify/specs/<feature>/spec.md`.
2. **Plan (`/speckit-plan`):** Include the constitution's **Constitution Check**, verifying explicitly that:
   1. Web rendering stays free of per-cell DOM elements (single `<canvas>`).
   2. Every interaction added works via **both keyboard and mouse**.
   3. Core logic and file serialization stay in the shared Rust core—no business rules in JavaScript or in the native terminal front end.
   4. The feature works on **both the web and native terminal targets**, or the plan documents the difference and why.
   5. Generated `.ods` files remain ODF 1.3 compliant, preserving data types, multiple sheets, and OpenFormula syntax.
   6. The 16 ms input-to-frame budget and the `.wasm` size measurement are addressed.
3. **Tasks (`/speckit-tasks`):** Break work into small, single-scope PRs respecting the L0–L6 layer graph.
4. **Implement & Verify:** Write code, behavioral tests, and run `cargo xtask` checks before opening a PR.

---

## 2. Recipe: Adding a New Command End-to-End

Every state-mutating action in Proto123 MUST be a registered `CommandId`. Never mutate workbook state directly from UI or web event handlers.

1. **Define the reversible operation (`L2` — `proto123-ops`):**
   - Implement the atomic mutation and its inverse `UndoEntry` delta on `Workbook` / `Sheet`.
   - If the operation shifts rows/columns or edits formulas, update references and return dirty cell coordinates for `proto123-calc`.
   - Add unit tests verifying both the mutation and `undo -> redo` round-trip.

2. **Register the `CommandId` (`L4` — `proto123-engine`):**
   - Add a stable, typed variant/ID (e.g., `range.sort`) to the command registry with parameter schema and undo metadata.
   - Wire execution to call `proto123-ops` + `proto123-calc` incremental recalculation.
   - Confirm no panics on invalid ranges or empty selections (return typed errors).

3. **Wire the Slash Command Bar (`L4` — `proto123-engine/src/menu_catalog.rs`):**
   - If the command belongs in the `/` menu, add a concise, trademark-safe label (see [`docs/feature-names.md`](feature-names.md)) with a unique single-letter hotkey within its submenu level.
   - Verify with `cargo xtask menu-commands`.

4. **Wire Keyboard & Mouse Interaction (`L5` — `proto123-tui`, shared by both targets):**
   - Map keyboard shortcuts / mode transitions in `InputMapper`.
   - Ensure mouse hit-testing (`LayoutMap`) supports hover highlight and click (and drag/double-click/wheel if applicable on the grid or headers) per [`docs/interaction-spec.md`](interaction-spec.md).
   - Keep UI copy inside the centralized English string catalog (`AGENTS.md` Rule 17).
   - Add headless `proto123-tui` tests exercising **both** keyboard and mouse paths, for web pixel input and native terminal cell input.
   - Do not add command logic to `apps/proto123-wasm` or `apps/proto123-term`; they only forward events and perform I/O.

5. **Expose to Headless CLI (`L6` — `apps/proto123-cli`):**
   - Ensure `proto123-cli exec` can invoke the new `CommandId` from JSON scripts.

6. **Update Parity Matrix:**
   - Update [`docs/parity-checklist.md`](parity-checklist.md) and run `cargo xtask parity`.

---

## 3. Recipe: Adding a New OpenFormula Function

1. **Check the Standard:** Consult the public **OASIS ODF 1.3 Part 4 (OpenFormula)** specification for argument types, edge cases, and error propagation (`#VALUE!`, `#NUM!`, `#DIV/0!`, `#N/A`). Never inspect GPL/LGPL/AGPL or proprietary spreadsheet code.
2. **Implement in `L1` (`proto123-formula`):**
   - Register the canonical uppercase OpenFormula function name in the evaluator registry.
   - Emit valid `of:=...` syntax in the canonical OpenFormula serializer.
   - Add exact-value unit tests covering normal inputs, empty cells, ranges, type coercion, and error propagation.
3. **Verify `.ods` Round-Trip (`L3` — `proto123-ods`):**
   - Confirm `write_ods -> read_ods -> recalc` preserves both the formula text and evaluated value.
   - Run `cargo xtask ods-conformance`.
4. **Update Documentation:** Record the function under [`docs/odf-conformance.md`](odf-conformance.md).

---

## 4. Clean-Room, License & Dependency Rules

- **Clean-room only:** Do NOT copy source code, assets, UI text, or test files from other spreadsheet applications, and do NOT study GPL/LGPL/AGPL spreadsheet source code.
- **Trademark-safe naming:** Follow [`docs/feature-names.md`](feature-names.md) for all user-visible labels, CLI flags, and docs headings.
- **Permissive crates only:** Any new crate added to `Cargo.toml` MUST use `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `Unicode-3.0`, or `MPL-2.0`, and MUST compile for `wasm32-unknown-unknown` (unless restricted to `apps/proto123-term`, `apps/proto123-cli`, or `xtask`).
- **Attribution:** Log any new test fixture (`corpus/`) or font (`web/fonts/`) in [`ATTRIBUTION.md`](../ATTRIBUTION.md) in the same PR.

---

## 5. Pre-PR Verification Checklist

Run the applicable commands for every crate you touched and report explicitly in your PR description which checks ran:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p <modified-crate>
cargo xtask layers
cargo xtask menu-commands
cargo xtask wasm
cargo build -p proto123-term
cargo xtask ods-conformance
```

If you modified `web/` or `Dockerfile`, also run:

```sh
cargo xtask docker-check
```

Before requesting review:
- [ ] PR addresses one focused issue or Spec Kit task (moves/renames separated from behavior changes).
- [ ] No `#[allow(...)]` lint suppressions or `unsafe` blocks added.
- [ ] No panics (`unwrap`/`expect`) introduced in non-test code.
- [ ] Keyboard **and** mouse interactions both tested for any UI change, on the web and native terminal targets.
- [ ] Crash/bug fixes include a regression test (and a hostile-input fixture in `corpus/hostile/` if applicable).
- [ ] No scratch files (`log/`, `plan/`, `.claude/`, `.cursor/`, `.codex/`) committed.