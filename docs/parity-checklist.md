# Classic123 Command & Input Parity Checklist

> **Last updated:** 2026-11-24 · **Generated / verified by:** `cargo xtask parity` & `cargo xtask menu-commands`
> **Policy Reference:** `AGENTS.md` Rules 14 & 22 · `.specify/memory/constitution.md` Principle III

Every state-mutating action in **Classic123** MUST be registered as a typed `CommandId` in `classic123-engine`. This checklist tracks parity across **Keyboard**, **Mouse**, **Slash Command Bar (`/`)**, and **Headless CLI** so no interaction path lags behind or becomes unreachable.

---

## Legend

- `[x]` Implemented and covered by automated tests
- `[~]` Partially implemented / in progress
- `[ ]` Planned (not yet implemented)
- `N/A` Intentionally not applicable to this surface (reason noted)

---

## 1. Navigation & Selection (`nav.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `nav.selectCell` | Move active cursor to cell `(col, row)` | Arrows / `Tab` / `Enter` | Single-click cell | `N/A` | `exec` | `[ ]` |
| `nav.moveRelative` | Move cursor by `(dcol, drow)` | Arrow keys / `Shift+Tab` | `N/A` | `N/A` | `exec` | `[ ]` |
| `nav.pageScroll` | Jump viewport by one page | `PgUp` / `PgDn` / `Alt+PgUp/Dn` | Mouse wheel / `Shift+Wheel` | `N/A` | `N/A` | `[ ]` |
| `nav.jumpHome` | Jump to `A1` or row start | `Home` / `Ctrl+Home` | `N/A` | `N/A` | `exec` | `[ ]` |
| `nav.jumpEdge` | Jump to contiguous data boundary | `End` then Arrow | `N/A` | `N/A` | `exec` | `[ ]` |
| `nav.gotoCell` | Prompt and jump to target address | `F5` | Click address box | `/Range/Goto` | `exec` | `[ ]` |
| `nav.extendSelection` | Select rectangular cell range | `Shift+Arrows` | Click-and-drag across cells | `N/A` | `exec` | `[ ]` |
| `nav.selectRow` | Select entire active row | `Shift+Space` | Click row number header | `N/A` | `exec` | `[ ]` |
| `nav.selectColumn` | Select entire active column | `Ctrl+Space` | Click column letter header | `N/A` | `exec` | `[ ]` |
| `nav.switchSheet` | Switch active worksheet tab | `Ctrl+PgUp` / `Ctrl+PgDn` | Click sheet tab in status bar | `/Worksheet/Switch` | `exec` | `[ ]` |

---

## 2. Cell Editing, Formula Entry & History (`cell.*`, `edit.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `cell.beginEdit` | Enter `EDIT` mode on active cell | `F2` | Double-click cell / click formula bar | `N/A` | `N/A` | `[ ]` |
| `cell.commitEdit` | Parse input, update DAG, return to `READY` | `Enter` / Arrow (in `VALUE`/`LABEL`) | Click confirm or another cell | `N/A` | `exec` | `[ ]` |
| `cell.cancelEdit` | Discard uncommitted input buffer | `Esc` | Click cancel / press `Esc` | `N/A` | `N/A` | `[ ]` |
| `cell.pointInsertRef` | Insert clicked/navigated ref while in `POINT` mode | Arrow / `Shift+Arrow` after operator | Click or drag cells during formula entry | `N/A` | `N/A` | `[ ]` |
| `cell.toggleAbsRef` | Cycle `A1` -> `$A$1` -> `A$1` -> `$A1` | `F4` (in `EDIT`/`POINT`) | `N/A` | `N/A` | `N/A` | `[ ]` |
| `edit.undo` | Revert last `classic123-ops` delta | `Ctrl+Z` / `Alt+F4` classic undo | Click `/Edit/Undo` | `/Edit/Undo` | `exec` | `[ ]` |
| `edit.redo` | Reapply undone `classic123-ops` delta | `Ctrl+Y` / `Ctrl+Shift+Z` | Click `/Edit/Redo` | `/Edit/Redo` | `exec` | `[ ]` |

---

## 3. Range Operations (`range.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `range.erase` | Clear values/formulas in selection | `Delete` / `Backspace` | Click `/Range/Erase` | `/Range/Erase` | `exec` | `[ ]` |
| `range.copy` | Copy selection (with relative ref shift) | `Ctrl+C` | Click `/Range/Copy` | `/Range/Copy` | `exec` | `[ ]` |
| `range.cut` | Move selection (updating dependents) | `Ctrl+X` | Click `/Range/Move` | `/Range/Move` | `exec` | `[ ]` |
| `range.paste` | Paste range at active cell | `Ctrl+V` | Click `/Range/Paste` | `/Range/Paste` | `exec` | `[ ]` |
| `range.setNumberFormat` | Apply General/Fixed/Currency/Percent/Date format | Menu shortcut | Hover & click submenu | `/Range/Format` | `exec` | `[ ]` |
| `range.setAlignment` | Set Left (`'`), Right (`"`), Center (`^`) label align | Prefix or menu | Hover & click submenu | `/Range/Label` | `exec` | `[ ]` |
| `range.sort` | Sort range by primary key column asc/desc | Menu shortcut | Hover & click dialog | `/Range/Sort` | `exec` | `[ ]` |

---

## 4. Worksheet & Grid Structure (`sheet.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `sheet.insertRow` | Insert row above active cell (shift refs) | Menu shortcut | Hover & click menu | `/Worksheet/Insert/Row` | `exec` | `[ ]` |
| `sheet.deleteRow` | Delete active row (shift refs / `#REF!`) | Menu shortcut | Hover & click menu | `/Worksheet/Delete/Row` | `exec` | `[ ]` |
| `sheet.insertColumn` | Insert column left of active cell | Menu shortcut | Hover & click menu | `/Worksheet/Insert/Column` | `exec` | `[ ]` |
| `sheet.deleteColumn` | Delete active column | Menu shortcut | Hover & click menu | `/Worksheet/Delete/Column` | `exec` | `[ ]` |
| `sheet.resizeColumn` | Adjust column width in character cells | Menu shortcut + Arrows | Drag column header divider | `/Worksheet/Column/Width` | `exec` | `[ ]` |
| `sheet.addSheet` | Append new blank worksheet | Menu shortcut | Click `[+]` tab in status bar | `/Worksheet/Sheet/Add` | `exec` | `[ ]` |
| `sheet.renameSheet` | Rename active worksheet tab | Menu shortcut | Double-click sheet tab | `/Worksheet/Sheet/Rename` | `exec` | `[ ]` |
| `sheet.deleteSheet` | Remove active worksheet | Menu shortcut | Hover & click menu | `/Worksheet/Sheet/Delete` | `exec` | `[ ]` |
| `sheet.setLockedHeaders` | Lock top rows / left columns while scrolling | Menu shortcut | Hover & click menu | `/Worksheet/Titles` | `exec` | `[ ]` |

---

## 5. File, ODF 1.3 & Server I/O (`file.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `file.newWorkbook` | Reset session to clean empty workbook | Menu shortcut | Hover & click menu | `/File/New` | `exec` | `[ ]` |
| `file.openLocal` | Load local `.ods` or `.csv` via file picker | `Ctrl+O` | Hover & click menu | `/File/Open` | `inspect`/`recalc` | `[ ]` |
| `file.fetchRemote` | Fetch `.ods`/`.csv` from URL via `fetch()` | Menu shortcut | Hover & click dialog | `/File/Fetch URL` | `N/A` (local path) | `[ ]` |
| `file.saveOds` | Serialize 100% ODF 1.3 `.ods` & download | `Ctrl+S` | Hover & click menu | `/File/Save ODS` | `recalc -o` | `[ ]` |
| `file.pushRemote` | Serialize `.ods` and send via `PUT`/`POST` hook | Menu shortcut | Hover & click menu | `/File/Push Server` | `N/A` | `[ ]` |
| `file.exportCsv` | Export active sheet to RFC 4180 `.csv` | Menu shortcut | Hover & click menu | `/File/Export CSV` | `convert` | `[ ]` |

---

## 6. Slash Menu, View & Calculation (`menu.*`, `view.*`, `calc.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `menu.open` | Enter `MENU` mode on top Slash Command Bar | `/` | Click menu trigger / top bar | *(root)* | `N/A` | `[ ]` |
| `menu.back` | Step back one menu level or return to `READY` | `Esc` | Click outside menu / back | *(navigation)* | `N/A` | `[ ]` |
| `view.cycleTheme` | Switch high-contrast TUI color theme | Menu shortcut | Hover & click menu | `/View/Theme` | `N/A` | `[ ]` |
| `view.toggleFormulas` | Toggle formula text vs evaluated values in grid | ``Ctrl+` `` | Hover & click menu | `/View/Show Formulas` | `N/A` | `[ ]` |
| `calc.recalculateAll` | Force full workbook DAG recalculation | `F9` | Hover & click menu | `/Worksheet/Recalc` | `recalc` | `[ ]` |
| `calc.validateOds` | Run in-memory ODF 1.3 structural self-check | Menu shortcut | Hover & click menu | `/File/Validate ODF` | `inspect` | `[ ]` |

---

## Maintenance Rules

1. Whenever a new `CommandId` is added to `classic123-engine`, add it to this table in the same PR and run `cargo xtask parity` and `cargo xtask menu-commands`.
2. Every entry in `menu_catalog.rs` MUST have a matching row here with both **Keyboard** and **Mouse** access verified.
3. Never mark a row `[x]` until unit/integration tests pass for both the engine behavior and (where applicable) TUI keyboard + mouse hit-testing in `classic123-tui`.