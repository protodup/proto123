# Feature Specification: Foundational MVP — Calculating Grid on a Single Canvas

**Feature Branch**: `001-foundational-mvp`

**Created**: 2026-11-25

**Status**: Draft

**Input**: User description: "Create the foundational vertical slice (MVP) of Proto123: an ultra-lightweight, classic-terminal-style TUI spreadsheet running in Rust + WebAssembly on a single HTML5 canvas with zero per-cell DOM elements, full hybrid keyboard + mouse ergonomics, and multi-stage Docker static deployment. Scope: (1) layered workspace with architecture guardrails (layer check, menu-command check), local serving and a multi-stage container image; (2) sparse spreadsheet core with A1 coordinates, typed values and label alignment prefixes, a formula language with arithmetic, references, and SUM/AVERAGE/MIN/MAX/COUNT, incremental dependency-graph recalculation with cycle detection and typed errors, and multi-step Undo/Redo; (3) a command-driven session with modes READY/VALUE/LABEL/EDIT/POINT/MENU, a single-canvas renderer with dirty-region redraw, high-contrast theme tokens and one accessibility live region, a two-line Slash Command Bar (/File New; /Range Erase, Copy, Move; /Edit Undo, Redo; /View Theme) operable by keyboard and mouse, and first-class mouse hit-testing (click, double-click, drag, wheel, formula-bar click). Out of scope (Spec 002): .ods and .csv file I/O, remote URL loading, and the headless CLI." Clarification Q1: the native terminal target ships in this slice too. (The full original text is quoted in the request that started this feature. Product trademarks in it are not used as feature names, per `docs/feature-names.md`.)

## Scope Summary

This is the first user-facing slice of Proto123. A person opens the app in a web browser and gets one in-memory worksheet. They can enter numbers, labels, and formulas. Dependent cells recalculate as they type. They can select ranges, erase, copy, and move them, and undo and redo their changes. They can do all of this with the keyboard alone or with the mouse. Maintainers get automated guardrails that keep the layered architecture and the command menu honest from the first commit. They also get a one-command local run and a container image for static hosting.

**In scope**: one worksheet per workbook, everything listed in the Requirements below, and both targets: the web app and a thin native terminal app for Linux, macOS, and Windows, including the minimal-terminal (Linux console) profile. Both targets run the same core and behave identically, except for the platform differences listed under FR-050. (Clarified 2026-11-25: Q1 → A.)

**Out of scope (deferred)**: saving or opening files (`.ods`, `.xlsx`, `.csv`), remote URL loading, push-to-server, the headless command-line tool, multiple worksheets and sheet tabs, number formats (Fixed, Currency, Percent, Date), column resizing, whole row/column selection from headers, row/column insert and delete, sort, clipboard-style cut/copy/paste (`Ctrl+C`/`Ctrl+X`/`Ctrl+V`, `Edit → Paste`), system clipboard integration, the absolute-reference toggle key, Show Formulas, Locked Headers, and the Help menu. These come in Spec 002 or later.

## Clarifications

### Session 2026-11-25

- Q: Should Copy and Move stay under `Range` as prompt-based commands (source range, then destination), with `docs/interaction-spec.md` §5 and `docs/feature-names.md` §4 updated to match? → A: Yes. `Range → Erase, Copy, Move` use source/destination prompts; both docs are updated to match; no `Ctrl+C`/`Ctrl+X`/`Ctrl+V` bindings in this slice.
- Q: When the Copy destination is a range larger than the source, should the source repeat to fill it or be pasted once at its top-left cell? → A: Tile. The source repeats in whole copies across the destination; a destination smaller than the source (or a single cell) means "paste once, anchored at its top-left".
- Q: In `VALUE` or `LABEL` mode, when no reference can be pointed at, what should the arrow keys do? → A: Commit the entry and move one cell in that direction. In `EDIT`, `Left`/`Right` move the caret, `Home`/`End` jump to the buffer's start or end, and `Up`/`Down` commit and move.
- Q: When a number's full value does not fit its 9-character column, how should it be displayed? → A: Round decimals to fit; if the integer part does not fit or a nonzero value would round to 0, use scientific form (`1.235E+11`, `1.2E-09`); if even that does not fit, fill the cell with `#`.
- Q: What counts as a plain number on entry, and what happens to digit-led text such as `123 Main St`? → A: A plain number is an optional sign, digits with an optional decimal point, and an optional exponent, with surrounding spaces trimmed. Any other digit-led entry is a formula; if it does not parse, the FR-015 diagnostic tells the user to start the entry with `'` to store it as text.
- Q: Should the engine calculate in decimal arithmetic (like a BigDecimal type) instead of binary floating point? → A: No. Numbers stay IEEE-754 double precision for interoperability with other ODF suites; the general display format is capped at 15 significant digits so binary artifacts never show (`=0.1+0.2` displays `0.3`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build a small calculating model with the keyboard (Priority: P1)

A person opens Proto123 (in a browser or in a terminal) and sees an empty grid with the active cell at `A1` and a `READY` mode badge. They type `120` into `A1` and `80` into `A2`, a label `Total` into `B3`, and `=SUM(A1:A2)` into `A3`. `A3` shows `200`. They change `A1` to `150`, and `A3` immediately shows `230`. They never touch the mouse.

**Why this priority**: Entering values and formulas and seeing correct, live results is the core of a spreadsheet. Every other story builds on it. On its own it is already a usable calculator grid.

**Independent Test**: Start a fresh session, perform the entries above by keyboard only, and check the displayed values, the mode badge after each keystroke, and the formula-bar content of each cell.

**Acceptance Scenarios**:

1. **Given** a fresh session in `READY` at `A1`, **When** the user types `1`, **Then** the mode badge shows `VALUE` and the control line shows the edit buffer `1` with a visible caret.
2. **Given** `VALUE` mode with buffer `120`, **When** the user presses `Enter`, **Then** `A1` stores the number 120, the active cell moves to `A2`, and the mode returns to `READY`.
3. **Given** `READY`, **When** the user types `T`, **Then** the mode shows `LABEL`, and committing `Total` stores a left-aligned text label.
4. **Given** `READY`, **When** the user types `^Total` and commits, **Then** the cell shows `Total` centered and the `^` is not shown as part of the text. `"` gives right alignment and `'` gives left alignment. In the formula bar, the stored label shows with its prefix.
5. **Given** `A1`=120 and `A2`=80, **When** the user enters `=SUM(A1:A2)` in `A3`, **Then** `A3` displays `200` and the formula bar shows `=SUM(A1:A2)` while `A3` is active.
6. **Given** the model above, **When** `A1` is changed to 150, **Then** `A3` displays `230` without any manual recalculation step.
7. **Given** a cell with content, **When** the user presses `F2`, **Then** the mode becomes `EDIT` with the existing raw content in the buffer and the caret at its end. `Esc` discards changes and restores the prior content. `Enter` commits.
8. **Given** `VALUE` with buffer `120` in `A1`, **When** the user presses `Right`, **Then** `A1` stores 120, `B1` becomes active, and the mode returns to `READY`. **Given** `EDIT` on the same cell, **When** the user presses `Left`, **Then** the caret moves one character left and nothing is committed.
9. **Given** `READY`, **When** the user presses the arrow keys, `Tab`/`Shift+Tab`, `Enter`/`Shift+Enter`, `PgUp`/`PgDn`, `Home`, or `Ctrl+Home`, **Then** the active cell moves as defined in `docs/interaction-spec.md` §3, and the viewport scrolls to keep the active cell visible.

---

### User Story 2 - Trust the results: errors, cycles, and correct math (Priority: P1)

A person builds formulas that divide by an empty cell, mistype a function name, reference beyond the grid, or accidentally refer back to themselves. Proto123 never shows a wrong number, never freezes, and never crashes. Each problem shows as a clear error token in the affected cell, and the status bar says why. Fixing the cause clears the error everywhere it spread.

**Why this priority**: A spreadsheet that shows a plausible wrong number is worse than no spreadsheet. Correct arithmetic and honest errors are as important as data entry. They share the P1 slice.

**Independent Test**: Enter the error-producing formulas listed in the scenarios and check the exact displayed tokens, how errors spread to dependents, and recovery after the fix. Check exact numeric results for the precedence examples.

**Acceptance Scenarios**:

1. **Given** `B1` is empty, **When** the user enters `=10/B1` in `C1`, **Then** `C1` shows `#DIV/0!`.
2. **Given** any cell, **When** the user enters `=SUMM(A1:A3)`, **Then** the cell shows `#NAME?`. The formula text is kept exactly as typed so the user can fix it with `F2`.
3. **Given** `A1` holds the label `abc`, **When** the user enters `=A1+1`, **Then** the cell shows `#VALUE!`.
4. **Given** `A1` contains `=B1+1` and `B1` contains `=A1+1`, **When** the second formula is committed, **Then** both cells show `#CIRC!` and the session stays responsive. **When** either formula is replaced by a constant, **Then** both cells show correct numeric results again.
5. **Given** `C1` contains `=A1*2` and `A1` shows an error, **Then** `C1` shows the same error token.
6. **Given** a fresh cell, **When** the user enters `=2+3*4`, `=(2+3)*4`, `=-2^2`, `=2^3^2`, and `=10/4`, **Then** the results are `14`, `20`, `4`, `64`, and `2.5`. The precedence and associativity follow OpenFormula: unary minus binds tighter than `^`, and `^` is left-associative.
7. **Given** a formula whose reference would fall outside the grid limits (for example after a copy shifts it past column 1 or row 1), **Then** the cell shows `#REF!`.
8. **Given** a syntactically malformed entry such as `=SUM(A1:`, **When** the user presses `Enter`, **Then** the entry is not committed. The session stays in the edit mode with the caret near the problem, and the status bar shows a syntax diagnostic. `Esc` abandons the entry.
9. **Given** any cell, **When** the user enters `=0.1+0.2`, **Then** the cell, and the value in the live-region announcement, show `0.3`, never `0.30000000000000004`. **When** the user enters `=1-0.9`, **Then** the cell shows `0.1`.
10. **Given** `READY`, **When** the user enters `123 Main St` and presses `Enter`, **Then** the entry is not committed, and the status-bar diagnostic says that starting it with `'` stores it as text. Entering `'123 Main St` stores a left-aligned label. Entering `1.5E3` stores the number 1500.

---

### User Story 3 - Select, erase, copy, and move ranges, with Undo and Redo (Priority: P2)

A person selects `A1:A3` with `Shift+Arrow` (or with `F8` and plain arrows), opens the Slash Command Bar with `/`, picks `Range → Copy`, and points at `B1` as the destination. The values and formulas are copied. The formulas' relative references shift so that `=SUM(A1:A2)` copied one column right becomes `=SUM(B1:B2)`. They then undo the copy, redo it, erase a range, and undo several steps back to an earlier state.

**Why this priority**: Range operations and undo turn a calculator grid into a working tool. They depend on P1 but are independently testable.

**Independent Test**: From a scripted starting grid, run each range command, then a sequence of Undo/Redo, and compare the full grid state (raw content and displayed values) after each step with the expected snapshots.

**Acceptance Scenarios**:

1. **Given** the active cell `A1`, **When** the user presses `Shift+Down` twice, **Then** the range `A1:A3` is highlighted with a non-color cue as well as shading. The formula bar shows `A1:A3` as the selection.
2. **Given** `READY`, **When** the user presses `F8`, **Then** the status bar shows `[EXT]`, and plain arrows extend the selection until `F8` or `Esc` ends extend mode. The selection is kept.
3. **Given** a selected range, **When** the user runs `/Range Erase` (or presses `Delete`), **Then** every cell in the range becomes empty and dependent formulas recalculate.
4. **Given** a selected range, **When** the user runs `/Range Copy`, **Then** Proto123 prompts for the source range (pre-filled with the selection) and then for the destination. The user can confirm each prompt by typing a reference, by pointing with the arrow keys, or by clicking or dragging on the grid.
5. **Given** `B1` contains `=A1*2`, **When** the user runs `/Range Copy` with source `B1` and destination `B2:B5`, **Then** `B2` to `B5` contain `=A2*2` to `=A5*2` (the source tiles to fill the destination, FR-019).
6. **Given** a copy of formulas, **Then** relative references shift by the copy offset, references marked absolute with `$` (for example `$A$1`, `A$1`, `$A1`) keep their absolute parts, and any reference shifted outside the grid becomes `#REF!`.
7. **Given** `/Range Move` from `A1:A2` to `C1`, **Then** the contents are relocated, the moved formulas keep pointing at the same cells they referred to before, and formulas elsewhere that referred to the moved cells now refer to their new location. A formula that referred to a destination cell that was overwritten shows `#REF!`.
8. **Given** a sequence of at least 5 mutating commands, **When** the user presses `Ctrl+Z` (or runs `/Edit Undo`) repeatedly, **Then** each step is reverted in reverse order, and all dependent values recalculate to their earlier results. `Ctrl+Y` (or `/Edit Redo`) re-applies them in order.
9. **Given** an undone step, **When** the user makes a new edit, **Then** the redo history is discarded.
10. **Given** nothing to undo, **When** the user requests Undo, **Then** nothing changes and the status bar says there is nothing to undo.

---

### User Story 4 - Operate everything with the mouse (Priority: P2)

A person who has never used a terminal spreadsheet clicks a cell to select it, double-clicks to edit, drags to select a range, scrolls with the wheel, clicks the formula bar to edit at a specific character, clicks `Range` in the always-visible root bar and then `Copy`, and while typing a formula clicks cells to insert references.

**Why this priority**: Mouse parity removes the learning barrier for people new to the classic keyboard flow. It is required by the constitution (Principle III) and builds on P1 and P2.

**Independent Test**: Feed scripted, timestamped raw pointer events (press, release, move, wheel) through the shared input layer. Assert the resulting commands and session state without a real browser. Then do a manual smoke test in a browser.

**Acceptance Scenarios**:

1. **Given** `READY`, **When** the user clicks cell `C5`, **Then** `C5` becomes the active cell. A `Shift+click` on `E9` selects `C5:E9`.
2. **Given** `READY`, **When** the user double-clicks a cell (two presses on the same cell within the double-click interval), **Then** that cell enters `EDIT` with the caret at the end of its content.
3. **Given** `READY`, **When** the user presses on `B2`, drags to `D6`, and releases, **Then** `B2:D6` is selected. Dragging beyond the grid edge auto-scrolls the viewport and extends the selection.
4. **Given** the grid, **When** the user turns the mouse wheel one notch, **Then** the view scrolls 3 rows. `Shift+wheel` or a horizontal gesture scrolls columns.
5. **Given** a cell with content, **When** the user clicks inside the formula bar at character position *n*, **Then** the session enters `EDIT` with the caret at position *n* (clamped to the content length).
6. **Given** `VALUE` or `EDIT` with buffer `=SUM(` (caret right after an operator, `(`, `,`, `;`, or `:`), **When** the user clicks `B2` or drags `B2:B9`, **Then** the mode becomes `POINT`, and the reference `B2` or `B2:B9` is inserted into the buffer and updates live during the drag.
7. **Given** `VALUE`, `EDIT`, or `LABEL` where a reference is not expected, **When** the user clicks another cell, **Then** the current entry is committed (or, if it is invalid, the syntax diagnostic of Story 2 scenario 8 applies), and the clicked cell becomes active.
8. **Given** any mode, **When** the pointer hovers over a Slash Command Bar item, **Then** the item is highlighted with a non-color cue, and the second menu line shows its description or submenu preview.
9. **Given** `READY`, **When** the user clicks `Edit` in the root bar and then `Undo`, **Then** the last change is undone and the mode returns to `READY`. Clicking outside an open menu closes it and changes no cells.

---

### User Story 5 - Navigate the Slash Command Bar by keyboard and switch themes (Priority: P2)

A person presses `/`. The two-line command bar shows `File  Edit  Range  View`, with the highlighted item described on the second line. They press `R`, then `E` to erase the current selection. Later they press `/`, `V`, `T` to cycle to the next theme, and the whole screen restyles at once.

**Why this priority**: The `/` menu is the product's signature keyboard flow. In this slice it is also the only way to reach some commands, such as Theme and File New.

**Independent Test**: Drive key sequences into the session. Assert the menu level, the highlighted item, the second-line text, the executed command, and the final mode. Separately, run the automated check that every menu entry maps to a live command.

**Acceptance Scenarios**:

1. **Given** `READY`, **When** the user presses `/`, **Then** the mode becomes `MENU`, the first item is highlighted, and the second line describes it.
2. **Given** `MENU` at the root, **When** the user presses `Left`/`Right`, **Then** the highlight moves and wraps around. `Enter` opens the highlighted submenu, or runs its command.
3. **Given** `MENU`, **When** the user presses an item's unique first letter, **Then** that item opens or runs immediately.
4. **Given** a submenu, **When** the user presses `Esc`, **Then** the menu goes up one level. At the root, `Esc` returns to `READY`.
5. **Given** the menu tree, **Then** it contains exactly: `File → New`; `Edit → Undo, Redo`; `Range → Erase, Copy, Move`; `View → Theme`. Every entry runs a live, registered command.
6. **Given** `/File New` on a workbook with content, **Then** a confirmation prompt appears (default answer: cancel). Confirming leaves an empty workbook and clears the undo history. On an empty workbook, New happens without a prompt.
7. **Given** `/View Theme`, **Then** the next built-in theme is applied to the whole screen. Repeating it cycles through all built-in themes and back to the first.

---

### User Story 6 - Screen-reader and low-vision access in the browser (Priority: P3)

A person using a screen reader moves around the grid and hears concise announcements such as "C4, formula =SUM(C1:C3), value 150, READY" when the active cell, the mode, the menu highlight, or an error changes. A low-vision user can rely on contrast and on non-color cues for every state.

**Why this priority**: Accessibility is mandatory. It is listed after the core flows because the flows have to exist before they can be announced.

**Independent Test**: Check the text sent to the single live region after scripted navigation, mode, menu, and error changes. Measure the contrast of every built-in theme's token pairs. Check that each state indicator has a non-color cue.

**Acceptance Scenarios**:

1. **Given** the web page, **Then** the page has exactly one drawing surface for the whole user interface and exactly one live region. It has no per-cell or per-menu-item document elements.
2. **Given** a cell move, a mode change, a menu highlight change, or a new error message, **Then** the live region receives one concise update with the cell address, the raw content or formula, the displayed value, the mode, and any status message, as applicable.
3. **Given** any built-in theme, **Then** text has at least a 4.5:1 contrast ratio, and active borders and controls have at least 3:1.

---

### User Story 7 - Maintainers run, ship, and stay within the architecture (Priority: P3)

A contributor clones the repository and starts the app locally with one documented command. They build a container image that serves the same static app with the correct WebAssembly content type. Their pull request is checked automatically: a lower layer depending on a higher one fails the check, and a menu entry without a live command fails the check.

**Why this priority**: These guardrails keep every later feature honest. They have no direct value to end users, but they are needed to ship and maintain the product.

**Independent Test**: Run the layer check on the clean workspace (it passes). Add a forbidden upward dependency in a scratch change (it fails and names the offending edge). Repeat with a menu entry that points at an unregistered command. Build the container image, request the WebAssembly asset, and check the response content type.

**Acceptance Scenarios**:

1. **Given** the clean workspace, **When** the architecture layer check runs, **Then** it passes. **When** any component depends on a component in a higher layer (or on an unregistered component), **Then** it fails and names the offending dependency.
2. **Given** the clean workspace, **When** the menu-command check runs, **Then** it passes. **When** a menu entry references a command that is not registered, **Then** it fails and names the entry.
3. **Given** a fresh checkout with the documented toolchain, **When** the developer runs the documented local-serve command, **Then** the app loads in a browser from localhost and works offline after loading.
4. **Given** the container image, **When** it serves the app, **Then** WebAssembly assets have the content type `application/wasm`, and the app loads and works with no backend.
5. **Given** a release build, **Then** the optimized, compressed size of the WebAssembly bundle is measured and reported.
6. **Given** a fresh checkout, **When** the developer runs the documented native run command on Linux, macOS, or Windows, **Then** the terminal app starts with an empty workbook, and the native build succeeds on all three platforms.

---

### User Story 8 - Use Proto123 in a native terminal, including the Linux console (Priority: P2)

A person on a server runs Proto123 in their terminal: a desktop terminal emulator, an SSH session, `tmux`, or the Linux virtual console with no graphical session. They get the same grid, modes, menu, formulas, and undo as in the browser. In a terminal that reports the mouse, clicking, double-clicking, dragging, hovering, and the wheel work exactly as on the web. In the Linux console there is no mouse, so they use only the keyboard. The app picks a 16-color theme and single-line box-drawing borders, and `F8` replaces `Shift+Arrow` for range selection. When they quit, or the app hits an internal error, the terminal is left exactly as it was before.

**Why this priority**: The constitution requires every feature to work on both targets. A terminal spreadsheet also has to work where only a text console exists. This story reuses the shared core and layout from P1 and P2.

**Independent Test**: Run the shared scenario suite (User Stories 1–5) against both targets and compare the resulting state. Render frames with the console profile into an off-screen test terminal and assert that only ANSI 16 colors and allowed glyphs appear. Before release, do a keyboard-only manual smoke test in a real Linux virtual console: edit, `/` menu, range selection, undo.

**Acceptance Scenarios**:

1. **Given** a terminal that reports the mouse, **When** the app starts, **Then** mouse reporting (button, drag, motion for hover, and wheel) is turned on, and the mouse scenarios of User Story 4 behave exactly as on the web.
2. **Given** the app is running, **When** the user quits normally, or the app exits because of an internal error, **Then** the terminal's normal screen, cursor, input mode, and mouse reporting are all restored.
3. **Given** `TERM=linux`, **When** the app starts, **Then** the console profile is chosen: the 16-color theme, single-line box-drawing glyphs, and no emphasis that depends on bold or underline. Every state indicator uses only glyphs, brackets, or inverted video that console fonts contain.
4. **Given** any detected profile, **When** the user overrides it (for example forcing the ASCII glyph set or another theme), **Then** the override takes effect.
5. **Given** a terminal that sends no mouse reports, **Then** every feature in this slice is still fully operable from the keyboard, and help and status text never assume a mouse.
6. **Given** the terminal window is resized, **Then** the frame is fully redrawn at the new size. Below 80×24, the "enlarge the window" message from the Edge Cases is shown.
7. **Given** a screen reader in the terminal, **Then** the mode badge, active cell, value, and status messages are plain text on the control line and status bar.

---

### Edge Cases

- **Huge ranges**: `=SUM(A1:XFD1048576)` on a sheet with 10 populated cells completes as fast as summing those 10 cells. Aggregate cost depends on the populated cells, not on the size of the range.
- **Far-corner cells**: Entering a value at the last row and column of the grid works, displays, and costs no more memory than any other single cell.
- **Out-of-grid references**: A typed reference beyond the grid limits (for example `XFE1` or row 1048577) is rejected as `#REF!` (or as a syntax error when it cannot be read as a reference at all). It is never silently clamped.
- **Cycles**: Self-references (`=A1` in `A1`), long cycles (`A1→A2→…→A500→A1`), and cycles inside ranges (`A5` = `=SUM(A1:A10)`) are all detected and shown as `#CIRC!` without unbounded recursion. Cells that depend on a cycle show `#CIRC!` too.
- **Deep chains**: A chain of 10,000 cells, each referencing the previous one, recalculates correctly without stack exhaustion.
- **Wide content**: A label longer than its column spills over empty neighbors to the right and is cut off at the first non-empty neighbor. A number too wide for its column is rounded or shown in scientific form (FR-006); if even that does not fit, it shows the `#` overflow marker filling the column, never truncated digits.
- **Numbers**: Results that are not finite (overflow to infinity, or not-a-number) show as `#VALUE!` (or `#DIV/0!` for division by zero) and never as `inf` or `NaN`. `-0` displays as `0`.
- **Aggregates over nothing**: `COUNT` of a range with no numbers is 0. `SUM` is 0. `MIN`/`MAX` are 0. `AVERAGE` is `#DIV/0!`.
- **Mixed ranges**: Aggregates skip text, empty, and boolean cells in referenced ranges. An error inside a referenced range spreads to the aggregate.
- **Small screens**: Below the 80×24 character minimum, the app shows a clear "enlarge the window" message instead of a broken layout. Keyboard and mouse input stays safe, and nothing is mutated by accident. Resizing back restores the full layout.
- **Terminal quirks**: Key combinations the terminal does not report (for example in the Linux console) are reachable another way (FR-024). Inside `tmux` or `screen` without mouse passthrough, the app behaves as a no-mouse terminal. Unknown or empty `TERM` values fall back to the most conservative profile: 16 colors and ASCII glyphs.
- **Resize and zoom**: When the browser resizes or the device pixel ratio changes, the frame is fully redrawn, and the mouse targets stay in line with what is drawn.
- **Input during prompts**: Pressing `Esc` at any prompt or menu level never leaves a half-applied command. Copy and Move are atomic, and Undo reverts each one as a single step.
- **Overlapping Copy/Move**: Copy and Move where the source and destination ranges overlap produce the same result as if the source had been fully captured first.
- **Undo after File New**: After a confirmed New, Undo has nothing to undo and says so.
- **Hostile input**: Extremely long formulas, deeply nested parentheses, and unexpected characters return a diagnostic or an error token. They never crash or freeze the session.

## Requirements *(mandatory)*

### Functional Requirements

**Grid, values, and entry**

- **FR-001**: The workbook MUST hold one worksheet with addressable cells from `A1` to `XFD1048576` (16,384 columns × 1,048,576 rows). Storage and processing cost MUST grow with the number of populated cells, not with the grid size.
- **FR-002**: Cell addresses MUST be readable and writable in A1 notation, with optional `$` absolute markers on the column and/or row. Column letters are not case-sensitive on input and are always shown in upper case.
- **FR-003**: Every cell value MUST be one of: Empty, Number, Text, Boolean, or Error. Error kinds are `#DIV/0!`, `#REF!`, `#NAME?`, `#VALUE!`, and `#CIRC!`.
- **FR-004**: An entry that parses entirely as a plain number MUST be stored as a Number. A plain number is, after trimming surrounding spaces, an optional `+` or `-` sign, one or more digits with at most one decimal point (`5`, `5.`, `.5`, `-12.75`), and an optional exponent `E` or `e` with an optional sign and one or more digits (`1.5E-3`). Thousands separators, percent signs, and currency symbols are not part of a plain number in this slice. An entry starting with `=`, `+`, `-`, `.`, `(`, `@`, or a digit that is not a plain number MUST be treated as a formula (`@` is the At-Prefix Shorthand for functions). Any other entry MUST be stored as Text.
- **FR-005**: A label prefix `'`, `"`, or `^` MUST set left, right, or center alignment. The prefix is shown in the formula bar but not in the grid. Labels without a prefix are left-aligned. Numbers are right-aligned.
- **FR-006**: Numbers MUST display in a general format within the column width, using the first of these that fits: (1) the value rounded to 15 significant digits, written without trailing zeros; (2) that form with decimals rounded to fit (`1/3` → `0.3333333`); (3) scientific form `d.dddE±XX` with as many mantissa digits as fit and trailing zeros dropped (`123456789012` → `1.235E+11`, `0.0000000012` → `1.2E-09`, while `0.000001234` → `0.0000012` at step 2), used when the integer part does not fit or a nonzero value would round to `0`; (4) the overflow marker, which fills the column with `#`. Rounding is half away from zero and affects display only: the stored value keeps full precision. No displayed number, in the grid, the formula bar, or the live region, ever shows more than 15 significant digits.

**Formulas and calculation**

- **FR-007**: Formulas MUST support number literals, the Boolean literals `TRUE` and `FALSE`, binary `+ - * / ^`, unary `-` and `+`, parentheses, single-cell references, rectangular range references, and the functions `SUM`, `AVERAGE`, `MIN`, `MAX`, and `COUNT`. Function names are not case-sensitive. Functions take one or more arguments, separated by `,` or `;`, and each argument is a value, a reference, or a range.
- **FR-008**: Operator precedence and associativity MUST follow OpenFormula. Unary prefix operators bind tighter than `^`. `^` binds tighter than `*` and `/`, which bind tighter than `+` and `-`. All binary operators are left-associative.
- **FR-009**: Arithmetic MUST treat Empty as 0 and Boolean as 1/0. Text MUST produce `#VALUE!`. Division by zero MUST produce `#DIV/0!`. An unknown function name MUST produce `#NAME?`. A reference outside the grid MUST produce `#REF!`. Any non-finite result MUST produce an error token, never a displayed infinity or not-a-number.
- **FR-010**: Errors MUST spread: any formula that uses an error value as an operand or argument (including through a referenced range) results in that error.
- **FR-011**: The aggregate functions MUST follow the rules in the Edge Cases: Text, Empty, and Boolean cells inside ranges are skipped; `AVERAGE` with no numbers is `#DIV/0!`; `MIN`, `MAX`, and `SUM` with no numbers are 0; `COUNT` counts numbers.
- **FR-012**: The formula bar MUST show formulas in a canonical form with a leading `=`. A formula entered with `+` or `@` is shown in that canonical form after commit, and its result is unchanged.
- **FR-013**: The system MUST track which cells each formula depends on. A commit MUST recalculate only the changed cell and the cells that depend on it, directly or indirectly, each exactly once and after the cells it depends on. Unrelated cells MUST NOT be re-evaluated.
- **FR-014**: Circular references MUST be detected on every change. Every cell in a cycle, and every cell that depends on one, MUST show `#CIRC!`. Breaking the cycle MUST restore correct values. Detection MUST terminate on cycles of any length.
- **FR-015**: A formula that cannot be parsed MUST NOT be committed. The session MUST stay in edit with a status-bar diagnostic that points at the problem position, and `Esc` MUST abandon the entry. When the entry did not start with `=`, `+`, or `@` (for example `123 Main St` or `1,000`), the diagnostic MUST also say that starting the entry with `'` stores it as text.

**Commands, modes, and undo**

- **FR-016**: Every change to workbook or session state MUST go through a registered, named command. Keyboard shortcuts, menu entries, and mouse gestures for the same action MUST run the same command.
- **FR-017**: The session MUST always be in exactly one of `READY`, `VALUE`, `LABEL`, `EDIT`, `POINT`, `MENU`, or `DIALOG` (`DIALOG` is used by the File New confirmation and the Copy/Move range prompts). The current mode MUST be shown as an uppercase text badge on the control line. Transitions MUST follow `docs/interaction-spec.md` §2.
- **FR-018**: Cell edits, Range Erase, Range Copy, and Range Move MUST each be recorded as one reversible step. The user MUST be able to undo and redo at least 100 steps in order. A new mutating command after an undo MUST discard the redo history. Navigation, selection, menu use, and theme changes MUST NOT create undo steps.
- **FR-019**: Range Copy MUST copy raw content and alignment, shifting relative references by the offset and keeping absolute parts. Any reference that would fall outside the grid MUST become `#REF!`. When the destination range is at least as large as the source along a dimension, the source MUST repeat along that dimension as many whole times as fit (a 1×3 source into `B1:B10` fills `B1:B9`; a single cell into `B1:B10` fills all ten), each copy shifting references by its own offset. Along a dimension where the destination is smaller than the source, or when the destination is a single cell, the source is pasted once, anchored at the destination's top-left cell. The whole copy is one undo step.
- **FR-020**: Range Move MUST relocate content without changing the targets of the moved formulas. It MUST redirect references elsewhere that pointed at the moved cells to their new location. References to destination cells that were overwritten MUST become `#REF!`. A range reference elsewhere MUST follow the move only when the whole referenced range lies inside the source; a range that only partly overlaps the source is left unchanged.
- **FR-021**: Copy and Move MUST give correct results when the source and destination ranges overlap.
- **FR-022**: File New MUST ask for confirmation when the workbook has content (with cancel as the default). After confirmation it MUST reset to an empty workbook and clear the undo and redo history.

**Keyboard**

- **FR-023**: Every feature in this slice MUST be fully operable from the keyboard with no focus traps. This includes the navigation keys of User Story 1, `Shift+Arrow` and `F8` extend mode for ranges, `F2` to edit, `Delete` to erase, `/` for the menu, `Esc` to cancel or step back, `Ctrl+Z`/`Ctrl+Y` for undo and redo, and arrow-key pointing in `POINT` mode (with `F8` to extend the live reference to a range). In `VALUE` and `LABEL`, an arrow key that does not start pointing (FR-029) MUST commit the entry and move the active cell one step in that direction. In `EDIT`, `Left`/`Right` MUST move the caret, `Home`/`End` MUST move it to the start or end of the buffer, and `Up`/`Down` MUST commit and move. A commit by arrow key follows FR-015 when the entry cannot be parsed. `docs/interaction-spec.md` §3 MUST be updated to list these bindings.
- **FR-024**: Every command bound to a key combination that some environments do not report (for example `Shift+Arrow` or `Ctrl+` combinations) MUST also be reachable through the Slash Command Bar or through an unmodified-key alternative such as `F8`.

**Slash Command Bar**

- **FR-025**: The Slash Command Bar MUST occupy two lines. The first line lists the items of the current level with the highlighted or hovered item marked. The second line shows that item's description or a preview of its submenu. In `READY`, a clickable root bar MUST stay visible.
- **FR-026**: The menu tree in this slice MUST be exactly: `File → New`; `Edit → Undo, Redo`; `Range → Erase, Copy, Move`; `View → Theme`. Each label MUST have a unique first letter within its level, and each leaf MUST run a registered command. `Range → Copy` and `Range → Move` are prompt-based (source range, then destination; FR-019, FR-020). This slice has no clipboard commands and no `Ctrl+C`/`Ctrl+X`/`Ctrl+V` bindings.
- **FR-027**: All menu text and all other user-facing strings MUST come from a central English string catalog.

**Mouse**

- **FR-028**: Pointer input MUST support: click to select a cell; `Shift+click` to extend the selection; double-click to edit; click-and-drag to select a range, with auto-scroll past the grid edge; wheel scrolling of 3 rows per notch; horizontal scrolling with `Shift+wheel` or a horizontal gesture; click on the formula bar to edit at the clicked character; hover highlight and click on every menu item and prompt control; and click outside an open menu to close it.
- **FR-029**: In `POINT`-eligible states (the edit buffer is a formula and the caret follows an operator, `(`, `,`, `;`, or `:`), a click or drag on the grid MUST insert or update the cell or range reference in the buffer. In other edit states, a click on another cell MUST commit the entry and select the clicked cell.
- **FR-030**: Double-click and drag MUST be worked out in the shared core from raw, timestamped press, release, and move events. This keeps behavior identical across hosts and testable without a browser. The double-click interval is 500 ms on the same cell.
- **FR-031**: Mapping a pointer position to the element under it MUST happen in two deterministic, unit-tested steps. First, map the surface pixel position to a character cell, taking device pixel ratio, font metrics, and scroll into account. Second, map the character cell to an element (grid cell, menu item, formula bar position, or prompt control) using the same layout pass that drew the current frame.

**Rendering, themes, and accessibility**

- **FR-032**: The web user interface MUST draw onto a single drawing surface using a monospace character grid. Cells, menus, and prompts MUST NOT be created as individual document elements.
- **FR-033**: After an input event, only the screen regions that changed MUST be redrawn. Full redraws are allowed only on resize, device-pixel-ratio change, and theme change.
- **FR-034**: Every color, border glyph, and character-cell layout metric (column width, minimum screen size) MUST come from theme tokens. On the web, the pixel size of one character cell is measured from the drawing backend's font and is not a theme token. The slice MUST ship at least three built-in themes: a default high-contrast theme, a 16-color theme limited to the standard ANSI 16 colors, and a monochrome ASCII-glyph theme. `/View Theme` MUST cycle through them.
- **FR-035**: Active cell, range selection, menu highlight, hover, extend mode (`[EXT]`), and errors MUST each have a non-color cue: inverted video, brackets, a text marker, or an error token.
- **FR-036**: The web host MUST have exactly one accessibility live region. It MUST receive concise text updates on cell move, mode change, menu highlight change, and new status or error message.
- **FR-037**: The web host MUST only start the application, forward raw keyboard, pointer, wheel, and resize events, and update the live region. It MUST contain no spreadsheet, layout, or input-interpretation rules.

**Delivery and guardrails**

- **FR-038**: The repository MUST provide an automated layer check that fails when a component depends on a component in a higher architecture layer (L0–L6 as defined in `AGENTS.md`), or when a component has not been assigned a layer.
- **FR-039**: The repository MUST provide an automated menu-command check that fails when any menu entry does not map to a registered, live command.
- **FR-040**: All components MUST forbid unsafe code. The shared core MUST build for both the WebAssembly target and native targets.
- **FR-041**: The repository MUST provide a documented single command to serve the app locally (offline after the first load), and a multi-stage container build that serves the static app with `application/wasm` for WebAssembly assets.
- **FR-042**: Every release build MUST measure and report the optimized, compressed size of the WebAssembly bundle.
- **FR-043**: Unsupported or invalid operations MUST never be reported as success. Each MUST produce an error token or a status-bar diagnostic. Non-test code MUST NOT abort on any user input.

**Native terminal target**

- **FR-044**: The slice MUST ship a native terminal app for Linux, macOS, and Windows. It runs the same core and the same layout as the web app, and it MUST NOT require a graphical session, X11, Wayland, or GPU libraries.
- **FR-045**: The native app MUST only run the terminal event loop, draw the shared layout to the terminal, and turn terminal events into the same raw, timestamped input the core gets on the web. It MUST contain no spreadsheet, layout, or gesture rules.
- **FR-046**: On startup, the native app MUST turn on terminal mouse reporting (button, drag, motion for hover, and wheel). On every exit, including exits caused by internal errors, it MUST restore the terminal's normal screen, cursor, input mode, and mouse reporting.
- **FR-047**: The native app MUST detect the terminal's capability profile at startup from its environment (`TERM`, `COLORTERM`) and pass it to the core. The core picks the matching theme and glyph set through theme tokens. The user MUST be able to override the detected profile. `TERM=linux` MUST select the console profile, and unknown profiles MUST fall back to 16 colors and ASCII glyphs.
- **FR-048**: The console profile MUST use only the standard ANSI 16 colors, while still meeting the contrast rules in FR-034 against the console's default palette. It MUST NOT rely on bold or underline for emphasis. It MUST draw borders with single-line box-drawing glyphs only, and the ASCII glyph set MUST always be available.
- **FR-049**: Both targets MUST pass the same automated scenario suite for User Stories 1–5, run against the shared core with each target's input translation.
- **FR-050**: The only allowed platform differences in this slice are these. The accessibility live region (FR-036) is web-only; on native, the same information is plain text on screen. The pixel-to-character mapping step of FR-031 is web-only, because terminals already report character positions. Mouse operation on native depends on the terminal reporting it (FR-046, User Story 8).

### Key Entities

- **Cell Address**: A column and row position in the grid, readable and writable in A1 notation, with optional absolute markers on each part.
- **Range**: A rectangle between two cell addresses, always normalized from top-left to bottom-right.
- **Cell**: The content at an address: raw input (number, label with alignment, or formula text) plus its current computed value.
- **Value**: Empty, Number, Text, Boolean, or Error (of a defined error kind).
- **Formula**: A parsed expression made of literals, operators, references, ranges, and function calls. It has a canonical text form and a set of referenced cells.
- **Dependency Graph**: For each formula cell, the cells it reads. It is used to order recalculation and to detect cycles.
- **Workbook / Worksheet**: The in-memory document. In this slice it is one sparse worksheet.
- **Command**: A named, registered action with typed parameters. It is the only way state changes.
- **Undo Step**: The minimal reversible change produced by one mutating command.
- **Session**: The current mode, active cell, selection, extend-mode flag, edit buffer and caret, menu position, prompt state, viewport scroll, theme, and status message.
- **Menu Entry**: A label, a description, a unique trigger letter, and either child entries or a bound command.
- **Theme**: A named set of color, glyph, and metric tokens.
- **Control Line**: The top screen line. It holds the address box (`[A1]`), the formula bar, and the mode badge. The formula bar shows the active cell's raw content, the selected range (`A1:A3`), or, while editing, the edit buffer with its caret.
- **Layout Map**: A record, produced by each frame's layout, of which element sits under each character cell.

## Constitution Check *(required by the constitution's Development Workflow & Quality Gates)*

1. **No per-cell document elements (I)**: FR-032 and FR-037, and US6 scenario 1. ✅
2. **Keyboard and mouse for every interaction (III)**: FR-023 to FR-031, and US4 and US5. ✅
3. **Logic stays in the shared core (II)**: FR-016, FR-030, FR-031, and FR-037. ✅
4. **Both targets (II)**: The web and native apps both ship (FR-044 to FR-049), and the allowed differences are listed in FR-050. ✅
5. **ODF compliance (IV)**: Not applicable to this slice, which has no file output. File I/O is deferred to Spec 002. The formula representation (FR-008, FR-012) follows OpenFormula, so the later round-trip is not blocked. ✅ (deferred)
6. **16 ms budget and WebAssembly size (I)**: SC-002 and FR-042. ✅
7. **Minimal-terminal profile (V)**: The 16-color and ASCII themes (FR-034), keyboard-only completeness, and non-Shift alternatives (FR-023, FR-024) are in scope, as are console capability detection and the console profile (FR-047, FR-048, User Story 8). ✅

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A first-time user can enter two numbers and a sum formula, change one number, and see the updated total in under 60 seconds, using the keyboard only or the mouse only.
- **SC-002**: On a reference worksheet of 10,000 populated cells (half of them formulas), every input event (keypress, click, drag step, wheel notch) produces its updated frame within 16 ms.
- **SC-003**: Changing one cell recalculates only that cell and its dependents. In a worksheet where a change affects 10 of 10,000 formulas, exactly 10 formulas are re-evaluated.
- **SC-004**: 100% of the formula examples in User Story 2 and the Edge Cases produce the exact expected value or error token.
- **SC-005**: Zero crashes, freezes, or hangs across the hostile-input sweep (long and nested formulas, cycles up to 10,000 cells, references at grid limits, random key and pointer sequences).
- **SC-006**: After any sequence of up to 100 mutating commands, undoing all of them restores a grid identical in raw content and displayed values to the starting grid. Redoing all of them restores the final grid.
- **SC-007**: 100% of the menu entries run a registered command, and 100% of the commands in this slice can be reached by both keyboard and mouse.
- **SC-008**: Every built-in theme meets 4.5:1 text contrast and 3:1 control contrast. Every state indicator stays distinguishable in the monochrome theme.
- **SC-009**: The web page holds exactly one drawing surface and one live region, whatever the sheet size or scroll position.
- **SC-010**: A new contributor can go from a fresh checkout to the app running locally with one documented command, and from there to a running container serving it, in under 15 minutes (excluding toolchain download time).
- **SC-011**: The layer check and the menu-command check each catch 100% of seeded violations in their verification tests.
- **SC-012**: 100% of the shared scenarios for User Stories 1–5 end in the same workbook and session state on the web and native targets.
- **SC-013**: In a keyboard-only manual test in a Linux virtual console, a tester completes the release smoke test (edit, `/` menu, range selection, undo) without a mouse and without leaving the app. After quitting, the console is fully usable.
- **SC-014**: In 100% of the tested exit paths (normal quit and internal error), the terminal is restored to its prior state.

## Assumptions

- **Grid limits** follow the common modern spreadsheet size (16,384 × 1,048,576), so later `.ods` and `.xlsx` import does not need to clip.
- **Default column width** is 9 characters. Column widths cannot be changed in this slice.
- **Operator semantics** follow OpenFormula (not older classic-spreadsheet precedence), so the formulas survive the `.ods` round-trip in Spec 002 unchanged. The `@` prefix is input shorthand only.
- **Number representation**: numbers are IEEE-754 double-precision values, the same model other ODF suites use, so a workbook calculates the same after the `.ods` round-trip in Spec 002. Decimal (BigDecimal-style) arithmetic was rejected for the core: it breaks that parity, cannot represent results such as `2^0.5` or `1/3` exactly anyway, and costs speed and bundle size. A dedicated exact decimal type (for example for currency) may come in a later spec. Binary rounding artifacts are hidden by the 15-significant-digit display cap (FR-006).
- **Booleans** come only from the `TRUE` and `FALSE` formula literals in this slice. Comparison operators and logical functions are deferred.
- **Undo depth** of at least 100 steps is enough for an in-memory session. History is lost when the page reloads, because persistence is deferred.
- **Menu placement** (clarified 2026-11-25): Copy and Move are prompt-based commands under `Range`, as the feature request asks. `docs/interaction-spec.md` §5 and §3 (which list them under `Edit` as a clipboard cut/paste pair bound to `Ctrl+C`/`Ctrl+X`/`Ctrl+V`) and `docs/feature-names.md` §4 MUST be updated to match in the same change that implements this spec. Clipboard-style commands are deferred.
- **Renderer choice**: How the shared layout is painted on the web is a planning decision. The constitution's default is the shared layout drawn by a canvas backend. A custom painter is allowed only if the plan shows the default cannot meet the 16 ms budget.
- **Copy/Move prompts** use the `DIALOG` mode and the canonical token list in `docs/feature-names.md` §3, even though the feature request lists only six modes.
- **Terminals**: the native app targets common terminal emulators on Linux, macOS, and Windows, plus the Linux virtual console. Native screen size, like the web canvas size, follows the 80×24 baseline. Native file open and save, the HTTP features, and system clipboard support (OSC 52) are deferred along with all other I/O.
- **Browsers**: current evergreen desktop browsers with WebAssembly support. Touch-only devices are not targeted in this slice.
- **No data leaves the device**: the slice has no network I/O and no persistence. A reload starts a new, empty session.
