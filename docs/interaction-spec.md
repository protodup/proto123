# Proto123 TUI & Hybrid Input Interaction Specification

> **Owning Crates:** `proto123-engine` (L4), `proto123-tui` (L5), `apps/proto123-wasm` and `apps/proto123-term` (L6)
> **Principles:** Zero DOM per cell · Keyboard-First Speed · First-Class Mouse Support · Same Behavior on Web and Native · Accessibility Without Cell DOM

This document defines the exact visual layout, mode transitions, keyboard shortcuts, Slash Command Bar (`/`) tree, and mouse hit-testing behavior for **Proto123**. Everything here applies identically to the **web target** (single `<canvas>`) and the **native terminal target**, unless a difference is explicitly listed. All interactions dispatch typed `CommandId`s defined in [`docs/parity-checklist.md`](parity-checklist.md).

---

## 1. Viewport Anatomy (80×24 Minimum Baseline)

Proto123 divides the character grid `(cols, rows)`—the `<canvas>` grid derived from `CellMetrics { char_width, char_height, dpr }` on the web, or the terminal size on native—into four fixed bands:

```text
Row 0:  [A1] (G) =SUM(B1:B10)                                            [READY]
Row 1:  File  Edit  Range  Worksheet  View  Help          (visible in MENU mode)
Row 2:  Open, save, fetch remote URL, or export .ods      (submenu hint / desc )
        ┌─────┬──────────┬──────────┬──────────┬──────────┬──────────┐
Row 3:  │     │    A     │    B     │    C     │    D     │    E     │
Row 4:  │   1 │   120.00 │    45.50 │          │          │          │
...     │   2 │    80.00 │    12.00 │          │          │          │
        └─────┴──────────┴──────────┴──────────┴──────────┴──────────┘
Row N-1: [Sheet1] [Sheet2] [+]   |  SUM=257.50  COUNT=4  |  / Menu · F1 Help
```

- **Row 0 — Control / Formula Line:**
  - Left: Active cell address (`[A1]`), optional format tag (`(G)`, `(C2)`, `(D)`), and raw cell input/formula text (or live edit buffer with visible cursor).
  - Right: Mode badge (`READY`, `VALUE`, `LABEL`, `EDIT`, `POINT`, `MENU`, `DIALOG`, `CALC`, `ERROR`).
  - Clicking Row 0 enters `EDIT` mode and places the text caret at the clicked character column; clicking `[A1]` opens `nav.gotoCell`.
- **Rows 1–2 — Slash Command Bar (Context Area):**
  - In `READY` mode: displays a compact clickable root bar (`[/] File  Edit  Range  Worksheet  View  Help`) on Row 1 so mouse users can click any menu directly without pressing `/` first, while Row 2 shows a subtle live summary or remains collapsed for extra grid rows (configurable).
  - In `MENU` mode: Row 1 shows the active menu level's options with highlighted active/hovered item; Row 2 shows either the next submenu preview or a concise description of the highlighted command.
- **Rows 3 .. N-2 — Grid Viewport:**
  - Column letter headers (`A`, `B`, ..., `AA`, ...) and row number gutter (`1`, `2`, ..., `1048576`).
  - Crisp Unicode box-drawing borders (with ASCII fallback theme) and clear inverted-video / bracket focus indicator for the active cell and shaded highlight for multi-cell range selections.
- **Row N-1 — Bottom Status & Sheet Tab Bar:**
  - Left: Clickable sheet tabs (`[Sheet1]`, `[Sheet2]`) and `[+]` button to add a sheet.
  - Center: Quick selection stats (`SUM`, `AVG`, `COUNT`) when a multi-cell numeric range is selected.
  - Right: Context hints or transient error/status diagnostics.

---

## 2. Mode State Machine (`SessionMode`)

Every keystroke and pointer event is interpreted deterministically according to the active mode (`READY`, `VALUE`, `LABEL`, `EDIT`, `POINT`, `MENU`, `DIALOG`):

```text
                ┌─────────────── (type digit = + - . ( @) ─────────► VALUE ──┐
                │                                                           │ (Arrow/click/drag
                ├─────────────── (type other printable / ' " ^) ───► LABEL   │  after operator)
                │                                                           ▼
  [READY] ──────┼─────────────── (F2 / dbl-click / formula-bar) ───► EDIT ◄─► POINT
     ▲          │                                                           │
     │ (Esc /   ├─────────────── ('/' or root-bar click) ──────────► MENU    │ (Enter/Tab/click
     │  commit) │                                                           │  commits)
     └──────────┴─────────────── (Copy/Move/New with content) ─────► DIALOG ─┘
```

Mode transitions:

1. **`READY` -> `VALUE`:** Typing a digit, `=`, `+`, `-`, `.`, `(`, or `@` starts replacing the active cell immediately.
2. **`READY` -> `LABEL`:** Typing any other printable character (other than `/`), including `'`, `"`, or `^`, starts a text label (`'`, `"`, `^` set left, right, or center alignment; shown in the formula bar, omitted from the grid).
3. **`READY` -> `EDIT`:** Pressing `F2`, double-clicking a cell, or clicking the formula bar edits the existing raw content in place.
4. **`VALUE` | `EDIT` -> `POINT`:** When the edit buffer is a formula and the caret immediately follows an operator (`=`, `+`, `-`, `*`, `/`, `^`), `(`, `,`, `;`, or `:`, an arrow key or grid click/drag enters `POINT` and inserts/updates a live reference (`B2` or `B2:D5`).
5. **`POINT` -> `VALUE` | `EDIT`:** Typing an operator or character returns to `VALUE` or `EDIT` while keeping the inserted reference.
6. **`VALUE` | `LABEL` -> `READY`:** An arrow key when not pointing commits the entry and moves one cell in that direction.
7. **`EDIT` -> `READY` | caret move:** `Up`/`Down` commit and move; `Left`/`Right`/`Home`/`End` move only the caret inside the buffer.
8. **`VALUE` | `LABEL` | `EDIT` | `POINT` -> `READY`:** `Enter`, `Tab`, or clicking elsewhere commits (`Esc` discards). On a parse error, the session stays in edit mode with a status-bar diagnostic.
9. **`READY` <-> `MENU`:** Pressing `/` or clicking a root-bar item opens `MENU`. `Esc` in a submenu steps up to the parent level; `Esc` at the root or clicking outside closes `MENU` back to `READY`.
10. **`MENU` -> `DIALOG`:** Selecting a leaf needing input (`Range -> Copy`, `Range -> Move`, or `File -> New` when the workbook has content) enters `DIALOG`.
11. **`DIALOG` -> `READY`:** `Enter` or clicking `[OK]`/`[Yes]` executes the command and returns to `READY`; `Esc` or clicking `[Cancel]`/`[No]` returns to `READY` with no change.

---

## 3. Keyboard Interaction Contract

Every feature MUST be 100% reachable by keyboard with zero focus traps, on both targets. Where a terminal or browser reserves a key (e.g., some terminals swallow `Shift+Arrow` or `Ctrl+`), the same command MUST stay reachable through the `/` menu or an unmodified alternative (`F8`).

| Context | Key(s) | Action / CommandId |
|---|---|---|
| `READY` | `Arrow Keys` | `nav.moveRelative` (1 cell) |
| `READY` | `Shift + Arrows` | `nav.extendSelection` (rectangular range) |
| `READY` | `F8` | `nav.toggleExtendMode`: anchors the selection at the active cell; plain arrows then extend it (`nav.extendSelection`) until `F8` or `Esc` ends the mode, keeping the selection. The status bar shows `[EXT]` while the mode is on |
| `READY` | `Tab` / `Shift+Tab` | `nav.moveRelative` right / left 1 cell |
| `READY` | `Enter` / `Shift+Enter` | `nav.moveRelative` down / up 1 cell |
| `READY` | `PgUp` / `PgDn` | `nav.pageScroll` vertical |
| `READY` | `Alt+PgUp` / `Alt+PgDn` | `nav.pageScroll` horizontal |
| `READY` | `Home` / `Ctrl+Home` | `nav.jumpHome` to column `A` of current row / `A1` |
| `READY` | `F2` | `cell.beginEdit` |
| `READY` | `Delete` | `range.erase` (`Backspace` is intentionally unbound in `READY`) |
| `READY` | `/` | `menu.open` (Slash Command Bar) |
| `READY` | `Ctrl+Z` / `Ctrl+Y` | `edit.undo` / `edit.redo` |
| `READY` (native) | `Ctrl+Q` | Quit `proto123-term` and restore terminal state |
| `VALUE` / `LABEL` | `Enter` / `Tab` / `Shift+Tab` | `cell.commitEdit` and move |
| `VALUE` / `LABEL` | `Arrow Keys` (not pointing) | `cell.commitEdit` and move 1 cell in arrow direction |
| `VALUE` / `LABEL` | `Home` / `End` / `Backspace` / `Delete` | `cell.editCaret` |
| `VALUE` / `LABEL` | `Esc` | `cell.cancelEdit` |
| `EDIT` | `Left` / `Right` / `Home` / `End` / `Backspace` / `Delete` | `cell.editCaret` (caret only) |
| `EDIT` | `Up` / `Down` / `Enter` / `Tab` / `Shift+Tab` | `cell.commitEdit` and move |
| `EDIT` | `Esc` | `cell.cancelEdit` |
| `VALUE` / `EDIT` (after operator, `(`, `,`, `;`, `:`) | `Arrow Keys` | Enter `POINT` and dispatch `point.setReference` |
| `POINT` | `Arrow Keys` / `Shift+Arrows` | `point.setReference` |
| `POINT` | `F8` | `nav.toggleExtendMode` (anchors the live reference so plain arrows extend it to `B2:D5`) |
| `POINT` | Printable character | Return to `VALUE`/`EDIT` keeping reference (`cell.editInput`) |
| `POINT` | `Enter` / `Tab` / `Esc` | `cell.commitEdit` / `cell.cancelEdit` |
| `MENU` | First letter of item | `menu.trigger` (open submenu or execute command) |
| `MENU` | `Left` / `Right` | `menu.move` (wraps around) |
| `MENU` | `Enter` | `menu.activate` |
| `MENU` | `Esc` | `menu.back` (up one level or exit to `READY`) |
| `DIALOG` | `Enter` / `Esc` / `Tab` / typing / arrows | `prompt.submit` / `prompt.cancel` / `prompt.focus` / `prompt.input` / `point.setReference` |

*(No `Ctrl+C`/`Ctrl+X`/`Ctrl+V` clipboard bindings exist in Spec 001; Copy and Move run via `/Range Copy` and `/Range Move`.)*

---

## 4. First-Class Mouse Interaction Contract (`LayoutMap`)

`proto123-tui` builds a `LayoutMap` (TUI cell rects → semantic `HitTarget`) during every layout pass, from the same layout that draws the frame—eliminating coordinate drift on resize or font change. Input reaches it in TUI cell units on both targets:

- **Web:** browser pixel coordinates `(x, y)` (scaled by `devicePixelRatio`) are converted by `proto123-tui::pixel_to_cell` using the current `CellMetrics`.
- **Native terminal:** `apps/proto123-term` enables mouse capture on start (SGR extended mouse reporting with press, release, drag, motion, and wheel events). The terminal already reports TUI cell coordinates. Inside `tmux`/`screen`, mouse passthrough must be enabled by the user; where the terminal reports no mouse events (for example the Linux virtual console, see §7), keyboard operation stays complete.
- **Gestures are derived in the shared core:** browsers and terminals both deliver raw presses, so `InputMapper` derives double-click (same cell within the double-click interval), drag, and hover from timestamped raw events. Both targets therefore behave identically and share the same tests.

### 4.1 Grid Mouse Controls
- **Single Click on Cell:**
  - In `READY`: dispatches `nav.selectCell(addr)`.
  - In `Shift + Click`: dispatches `nav.extendSelection(addr)` from the anchor cell.
  - In `VALUE`/`EDIT`/`POINT` (after formula operator): inserts/updates the clicked cell reference in the formula buffer.
  - In `VALUE`/`EDIT`/`LABEL` (not expecting reference): commits the active edit and selects the clicked cell.
- **Double Click on Cell:** Dispatches `cell.beginEdit(addr)` and places the text caret at the end of the cell content.
- **Click-and-Drag Across Cells:**
  - In `READY`: continuously dispatches `nav.extendSelection(addr)` to select a rectangular range. When the pointer moves near or beyond the grid edge while dragging, the viewport auto-scrolls smoothly.
  - In `POINT`: continuously updates the live range reference (e.g., `B2:E9`) inside the formula bar.
- **Mouse Wheel:**
  - Vertical wheel: scrolls rows (`view.scroll(0, dy)`, default 3 rows per notch).
  - Horizontal trackpad gesture, tilt wheel, or `Shift + Wheel`: scrolls columns (`view.scroll(dx, 0)`).
- **Column & Row Headers:**
  - Click column letter (`A`, `B`, ...) -> `nav.selectColumn(col)`.
  - Click row number (`1`, `2`, ...) -> `nav.selectRow(row)`.
  - Hover over the vertical border between two column headers -> the divider is highlighted in the TUI (and on the web the host also switches the cursor to `col-resize`); click-and-drag resizes width in character columns (`sheet.resizeColumn`), with live preview.

### 4.2 Slash Command Bar & Dialog Mouse Controls
- **Hover on Menu Item:** Immediately highlights the hovered item and updates Row 2 with its description or submenu preview (`hover_menu_item`).
- **Single Click on Root Menu Item (even from `READY`):** Opens `MENU` mode directly into that top-level menu's submenu.
- **Single Click on Submenu Item:** Either descends to the next submenu level or immediately executes the bound `CommandId` and returns to `READY`.
- **Click Outside Open Menu:** Dismisses `MENU` cleanly without mutating cells.
- **Dialogs:** Clicking inside a text field focuses/positions the caret; hovering highlights `[OK]` / `[Cancel]` buttons; clicking executes or cancels.

### 4.3 Formula Bar & Sheet Tabs
- **Formula Bar Click:** Focuses `EDIT` mode and positions the caret at the exact character column clicked.
- **Sheet Tab Click:** Switches active worksheet (`nav.switchSheet`).
- **Sheet Tab Double-Click:** Opens rename dialog (`sheet.renameSheet`).
- **`[+]` Tab Click:** Appends a new blank worksheet (`sheet.addSheet`).

---

## 5. Slash Command Bar (`/`) Hierarchy

All labels follow [`docs/feature-names.md`](feature-names.md), use unique single-letter triggers per level, and map to registered `CommandId`s checked by `cargo xtask menu-commands` (Spec 001 exact tree):

- **`File`** (`F`)
  - `New` (`N`) -> `file.newWorkbook`
- **`Edit`** (`E`)
  - `Undo` (`U`) -> `edit.undo`
  - `Redo` (`R`) -> `edit.redo`
- **`Range`** (`R`)
  - `Erase` (`E`) -> `range.erase`
  - `Copy` (`C`) -> `range.copy`
  - `Move` (`M`) -> `range.move`
- **`View`** (`V`)
  - `Theme` (`T`) -> `view.cycleTheme`

---

## 6. Clipboard & Accessibility Guarantees

- **Clipboard Interop:** `Ctrl+C` / `Ctrl+X` copies internal cell/formula payloads (preserving relative reference shifting inside Proto123). On the web it also writes RFC-compliant TSV to the browser clipboard; on native it writes TSV to the system clipboard through the terminal (OSC 52) where the terminal supports it, and the internal clipboard always works. Pasting external TSV/CSV (browser paste or terminal bracketed paste) parses cleanly into the target range.
- **High Contrast & Non-Color Cues:** Default TUI themes maintain at least **4.5:1** text contrast and **3:1** border/focus contrast. Active cell, selection range, hovered menu item, and error states always include explicit structural cues (inverted background, brackets, mode badge, error token)—never color alone.
- **Native terminal:** Mode badge, active cell, value, and status messages are plain text in the control line and status bar, so terminal screen readers can read them; a monochrome/ASCII theme covers limited terminals (see §7).

---

## 7. Minimal Terminals (Linux Console Without a Graphical Session)

The native target MUST be fully usable without X11 or Wayland: in the Linux virtual console (VT, `TERM=linux`), over SSH, inside `tmux`/`screen`, and in containers. This is a supported and tested environment, not best effort.

- **Capability profile:** `apps/proto123-term` detects the terminal's capabilities at startup (`TERM`, `COLORTERM`) and passes them to the core, which selects the matching theme and glyph set through theme tokens. The user MUST be able to override the detected profile. With `TERM=linux`, the console profile below is the default.
- **Colors:** A 16-color theme MUST exist that uses only the standard ANSI 16 colors, never 256-color or truecolor, and still meets the contrast rules in §6 against the console's default palette. It MUST NOT depend on bold or underline for emphasis, because the console shows them as color changes rather than as font styles.
- **Glyphs:** The console profile MUST draw borders using only single-line box-drawing characters (horizontal, vertical, corners, tees, cross), which common console fonts include, and an ASCII glyph set (`+`, `-`, `|`) MUST always be available. Every indicator of state (active cell, selection, hover, mode) MUST be drawn with those glyphs, brackets, or inverted video, never with symbols that are absent from console fonts (arrows, check marks, emoji, wide characters). Cell contents are never changed; a character the console font lacks is shown however the terminal renders it.
- **No mouse:** The Linux console does not send mouse reports to applications, and Proto123 does not link `libgpm`. Every feature MUST work by keyboard alone; help text and status hints MUST NOT assume a mouse is available.
- **Limited key combinations:** The Linux console does not report some modified keys (for example `Shift+Arrow`, `Ctrl+PgUp`, ``Ctrl+` ``). Every command bound to such a combination MUST also be reachable through the `/` menu, and range selection MUST be possible without `Shift`+arrows: `F8` (`nav.toggleExtendMode`, §3) anchors the selection in `READY` or the reference in `POINT`, and plain arrows then extend it.
- **Clipboard:** With no graphical session there is no system clipboard and the console does not support OSC 52. The internal clipboard MUST keep working, and the status bar MUST NOT report that data reached the system clipboard when it did not.
- **Screen size:** The console may be 80×25 or larger depending on the framebuffer; the 80×24 baseline in §1 covers it.
- **Verification:** Automated tests render frames with the console profile into a Ratatui test backend and assert that the UI chrome uses only ANSI 16 colors and glyphs from the allowed sets. Before each release, a manual smoke test runs `proto123-term` in a real Linux virtual console (open, edit, `/` menu, select a range, save `.ods`) using only the keyboard.
- **Screen Reader Bridge (web, Zero Cell DOM):** A single hidden `<div aria-live="polite" class="sr-only">` in `web/index.html` receives concise status updates from WASM on cell move, mode switch, menu highlight, and error notification (e.g., `"C4, formula =SUM(C1:C3), value 150, READY"`).