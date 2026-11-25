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

Every keystroke and pointer event is interpreted deterministically according to the active mode:

```text
                ┌─────────────── (type digit / '=' / '+' / '-') ──► VALUE ──┐
                │                                                           │ (Arrow/Mouse
                ├─────────────── (type letter / ' / " / ^) ───────► LABEL   │  after operator)
                │                                                           ▼
  [READY] ──────┼─────────────── (F2 / double-click / click bar) ─► EDIT ◄─► POINT
     ▲          │                                                           │
     │ (Esc /   ├─────────────── ('/' or click top menu item) ────► MENU    │ (Enter commits)
     │  commit) │                                                           ▼
     └──────────┴─────────────── (command needs text/confirm) ────► DIALOG ─┘
```

1. **`READY` -> `VALUE`:** Typing `0-9`, `=`, `+`, `-`, `.`, `(`, or `@` starts replacing the active cell immediately.
2. **`READY` -> `LABEL`:** Typing any other printable character starts a text label. Leading `'`, `"`, or `^` sets left, right, or center alignment respectively (stored in cell style, not displayed as raw text).
3. **`READY` -> `EDIT`:** Pressing `F2`, double-clicking a cell, or clicking the formula bar edits the existing raw content in place.
4. **`VALUE` / `EDIT` -> `POINT`:** When the edit buffer starts with `=` (or `@`) and the cursor immediately follows an operator, `(`, `,`, `;`, or `:`, pressing an arrow key or clicking/dragging on the grid switches to `POINT` mode and inserts/updates a live reference (`B2` or `B2:D5`). Typing another operator returns to `VALUE`/`EDIT` while keeping the reference.
5. **`MENU`:** Pressing `/` or clicking any top-level menu item activates `MENU`. `Esc` steps back one submenu level (or returns to `READY` from root). Clicking outside the menu area dismisses `MENU` cleanly.
6. **`DIALOG`:** Modal TUI overlay centered on the screen for prompts (e.g., *Fetch Remote URL*, *Open File*, *Save Filename*, *Rename Sheet*, *Confirm Clear*). `Enter` submits, `Esc` cancels, and mouse clicks work on input fields and `[OK]` / `[Cancel]` buttons.

---

## 3. Keyboard Interaction Contract

Every feature MUST be 100% reachable by keyboard with zero focus traps, on both targets. Where a terminal or browser reserves a key (e.g., some terminals swallow `Ctrl+PgUp`, browsers reserve `Ctrl+W`), the same command MUST stay reachable through the `/` menu.

| Context | Key(s) | Action / CommandId |
|---|---|---|
| `READY` | `Arrow Keys` | `nav.moveRelative` (1 cell) |
| `READY` | `Shift + Arrows` | `nav.extendSelection` (rectangular range) |
| `READY` | `Tab` / `Shift+Tab` | Move right / left 1 cell |
| `READY` | `Enter` / `Shift+Enter` | Move down / up 1 cell |
| `READY` | `PgUp` / `PgDn` | `nav.pageScroll` vertical |
| `READY` | `Alt+PgUp` / `Alt+PgDn` | `nav.pageScroll` horizontal |
| `READY` | `Home` / `Ctrl+Home` | Jump to column `A` of current row / `A1` |
| `READY` | `End` then `Arrow` | `nav.jumpEdge` (jump to contiguous data edge) |
| `READY` | `F2` | `cell.beginEdit` |
| `READY` | `F5` | `nav.gotoCell` |
| `READY` | `F9` | `calc.recalculateAll` |
| `READY` | `Delete` / `Backspace` | `range.erase` |
| `READY` | `/` | `menu.open` (Slash Command Bar) |
| `READY` | `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | `range.copy` / `range.cut` / `range.paste` |
| `READY` | `Ctrl+Z` / `Ctrl+Y` | `edit.undo` / `edit.redo` |
| `READY` | `Ctrl+S` / `Ctrl+O` | `file.saveOds` / `file.openLocal` |
| `READY` | `Ctrl+PgUp` / `Ctrl+PgDn` | `nav.switchSheet` previous / next |
| `READY` | ``Ctrl+` `` | `view.toggleFormulas` |
| `EDIT`/`VALUE`/`LABEL` | `Enter` | `cell.commitEdit` and move down |
| `EDIT`/`VALUE`/`LABEL` | `Tab` / `Shift+Tab` | `cell.commitEdit` and move right / left |
| `EDIT`/`VALUE`/`LABEL` | `Esc` | `cell.cancelEdit` |
| `EDIT`/`POINT` | `F4` | `cell.toggleAbsRef` (`A1` -> `$A$1` -> `A$1` -> `$A1`) |
| `MENU` | First letter of item | Jump directly to submenu or execute command |
| `MENU` | `Left` / `Right` + `Enter` | Highlight adjacent item / confirm |
| `MENU` | `Esc` | `menu.back` (up one level or exit to `READY`) |

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

All labels follow [`docs/feature-names.md`](feature-names.md), use unique single-letter triggers per level, and map to registered `CommandId`s checked by `cargo xtask menu-commands`:

- **`File`**
  - `New` (`file.newWorkbook`)
  - `Open` (`file.openLocal`) — `.ods`, `.xlsx`, or `.csv`
  - `Fetch URL` (`file.fetchRemote`) — `.ods`, `.xlsx`, or `.csv`
  - `Save ODS` (`file.saveOds`)
  - `Push Server` (`file.pushRemote`)
  - `Validate ODF` (`calc.validateOds`)
- **`Edit`**
  - `Undo` (`edit.undo`)
  - `Redo` (`edit.redo`)
  - `Copy` (`range.copy`)
  - `Move` (`range.cut`)
  - `Paste` (`range.paste`)
- **`Range`**
  - `Erase` (`range.erase`)
  - `Format` -> `General` | `Fixed` | `Currency` | `Percent` | `Date` (`range.setNumberFormat`)
  - `Label` -> `Left` | `Right` | `Center` (`range.setAlignment`)
  - `Sort` (`range.sort`)
  - `Goto` (`nav.gotoCell`)
- **`Worksheet`**
  - `Insert` -> `Row` (`sheet.insertRow`) | `Column` (`sheet.insertColumn`)
  - `Delete` -> `Row` (`sheet.deleteRow`) | `Column` (`sheet.deleteColumn`)
  - `Column` -> `Width` (`sheet.resizeColumn`)
  - `Sheet` -> `Add` (`sheet.addSheet`) | `Rename` (`sheet.renameSheet`) | `Delete` (`sheet.deleteSheet`)
  - `Titles` -> `Both` | `Horizontal` | `Vertical` | `Clear` (`sheet.setLockedHeaders`)
  - `Recalc` (`calc.recalculateAll`)
- **`View`**
  - `Theme` (`view.cycleTheme`)
  - `Show Formulas` (`view.toggleFormulas`)
- **`Help`**
  - `Shortcuts` | `Mouse Controls` | `About`

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
- **Limited key combinations:** The Linux console does not report some modified keys (for example `Shift+Arrow`, `Ctrl+PgUp`, ``Ctrl+` ``). Every command bound to such a combination MUST also be reachable through the `/` menu, and range selection MUST be possible without `Shift`+arrows (the plain-key alternative is defined together with the selection feature and listed in [`docs/parity-checklist.md`](parity-checklist.md)).
- **Clipboard:** With no graphical session there is no system clipboard and the console does not support OSC 52. The internal clipboard MUST keep working, and the status bar MUST NOT report that data reached the system clipboard when it did not.
- **Screen size:** The console may be 80×25 or larger depending on the framebuffer; the 80×24 baseline in §1 covers it.
- **Verification:** Automated tests render frames with the console profile into a Ratatui test backend and assert that the UI chrome uses only ANSI 16 colors and glyphs from the allowed sets. Before each release, a manual smoke test runs `proto123-term` in a real Linux virtual console (open, edit, `/` menu, select a range, save `.ods`) using only the keyboard.
- **Screen Reader Bridge (web, Zero Cell DOM):** A single hidden `<div aria-live="polite" class="sr-only">` in `web/index.html` receives concise status updates from WASM on cell move, mode switch, menu highlight, and error notification (e.g., `"C4, formula =SUM(C1:C3), value 150, READY"`).