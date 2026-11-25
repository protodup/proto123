# Backlog

Work that has been identified but not scheduled into a spec yet. When a spec takes an item, move it there and delete it here.

Each item records where it came from, what is done today instead, and a possible approach.

---

## B1 — Automated check: every modified-key command has another path

- **Source**: Spec 001 cross-artifact analysis, finding F15 (2026-11-25). Requirement FR-024; constitution Principle V (minimal terminals).
- **Today**: checked by hand through `docs/parity-checklist.md` (Spec 001 task T118).
- **Gap**: a new binding such as `Ctrl+…` or `Shift+…` with no `/` menu path and no unmodified-key alternative would pass CI. In the Linux console that command would be unreachable.
- **Possible approach**: mark each key binding in the command registry as modified or unmodified. Have `cargo xtask menu-commands` (or the planned `cargo xtask parity`) fail when a command has only modified bindings and no menu leaf. Add seeded-violation tests.

## B2 — Automated check: no user-facing strings outside the string catalog

- **Source**: Spec 001 analysis, finding F15. Requirement FR-027; AGENTS.md rule 17.
- **Today**: enforced only by review. Spec 001 tasks say every string goes into `crates/proto123-engine/src/strings.rs`.
- **Gap**: a literal such as `"Nothing to undo"` written straight into `render.rs` or an app crate would pass CI.
- **Possible approach**: an `xtask` check that scans `proto123-tui`, `proto123-engine` (except `strings.rs`), and the app crates for string literals reaching the frame buffer or status. It would need an allow-list for format strings, test code, and non-UI text such as log lines. Weigh the false-positive rate before making it blocking.

## B3 — Automated check: the web app works offline after the first load

- **Source**: Spec 001 analysis, finding F15. Requirement FR-041; User Story 7 scenario 3; constitution Principle V.
- **Today**: manual check in `.specify/specs/001-foundational-mvp/quickstart.md` (Spec 001 task T122).
- **Gap**: a change that loads a resource at runtime (a font, a script, a lazy WASM chunk) would break offline use without failing CI.
- **Possible approach**: a headless-browser test in CI that loads the page from the Docker image, cuts the network, then drives a short keyboard scenario and reads the live region. Adding a headless browser to CI is a tooling decision of its own. A cheaper first step is to check that `dist/` references no external URLs.

## B4 — Web themes with their own font size

- **Source**: Spec 001 analysis, finding F2 follow-up (2026-11-25). Requirement FR-034; AGENTS.md rule 14.
- **Today**: the font is a theme token (`GridMetrics.font_px`, `font_family`, `cell_px`, `origin_px`), but every built-in theme must use Ratzilla 0.3.1's fixed values: `16px monospace`, 10×19 CSS px cells, 5 px origin. The wasm app reports a mismatch as an error instead of drawing with other values.
- **Gap**: a user cannot pick a larger font (for low vision) or a different monospace family on the web. Browser zoom is the only workaround.
- **Possible approaches**, from smallest to largest:
  1. Contribute font-size, font-family, and cell-size options to Ratzilla's `CanvasBackendOptions` upstream, then pass the theme tokens through.
  2. Pin a small fork of Ratzilla's canvas backend with those options.
  3. A custom Canvas 2D painter over the same Ratatui buffer. The constitution allows this only if the plan shows Ratzilla cannot meet the 16 ms budget, so it is not justified by fonts alone.
- **Note**: a different font family needs either a font every target browser has or a bundled font with an approved license recorded in `ATTRIBUTION.md` (AGENTS.md rule 26).
