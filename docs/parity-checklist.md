# Proto123 Command & Input Parity Checklist

> **Last updated:** 2026-11-25 · **Verified by:** `cargo xtask menu-commands` (Spec 001 hand-updated matrix; `cargo xtask parity` lands in Spec 002)
> **Policy Reference:** `AGENTS.md` Rules 14 & 22 · `.specify/memory/constitution.md` Principle III

Every state-mutating action in **Proto123** MUST be registered as a typed `CommandId` in `proto123-engine`. This checklist tracks parity across **Keyboard**, **Mouse**, **Slash Command Bar (`/`)**, and **Headless CLI** so no interaction path lags behind or becomes unreachable.

**Both targets:** the **Keyboard**, **Mouse**, and **`/` Menu** columns apply to the web target *and* the native terminal target. A row is `[x]` only when both targets pass.

---

## Legend

- `[x]` Implemented and covered by automated tests (Spec 001)
- `[ ]` Planned (Spec 002+)
- `N/A` Intentionally not applicable to this surface

---

## 1. Navigation & Selection (`nav.*`, `view.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `nav.moveRelative` | Move cursor by `(dcol, drow)` | Arrows / `Tab` / `Shift+Tab` / `Enter` / `Shift+Enter` | `N/A` | `N/A` | Spec 002 | `[x]` |
| `nav.selectCell` | Move active cursor to cell `(col, row)` | via `moveRelative` | Single-click cell | `N/A` | Spec 002 | `[x]` |
| `nav.extendSelection` | Select rectangular cell range | `Shift+Arrows` / Arrows in `[EXT]` (`F8`) | `Shift+click` / click-and-drag | `N/A` | Spec 002 | `[x]` |
| `nav.toggleExtendMode` | Toggle `[EXT]` selection extension mode | `F8` (`Esc` ends) | `N/A` (drag selects directly) | `N/A` (`F8` reaches every target) | `N/A` | `[x]` |
| `nav.pageScroll` | Jump viewport by one page | `PgUp` / `PgDn` / `Alt+PgUp/Dn` | `N/A` | `N/A` | `N/A` | `[x]` |
| `nav.jumpHome` | Jump to row start or `A1` | `Home` / `Ctrl+Home` | `N/A` | `N/A` | Spec 002 | `[x]` |
| `view.scroll` | Scroll viewport without moving active cell | `N/A` | Wheel (3 rows) / `Shift+wheel` / edge drag | `N/A` | `N/A` | `[x]` |

---

## 2. Entry, Editing & Pointing (`cell.*`, `point.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `cell.beginEntry` | Start `VALUE` or `LABEL` entry from `READY` | Printable key (except `/`) | `N/A` | `N/A` | `N/A` | `[x]` |
| `cell.beginEdit` | Enter `EDIT` or reposition caret | `F2` | Double-click cell / click formula bar | `N/A` | `N/A` | `[x]` |
| `cell.editInput` | Insert character into edit buffer | Printable key in edit/`POINT` | `N/A` | `N/A` | `N/A` | `[x]` |
| `cell.editCaret` | Move caret or delete char in edit buffer | `Left`/`Right`/`Home`/`End`/`Backspace`/`Delete` | `N/A` | `N/A` | `N/A` | `[x]` |
| `cell.commitEdit` | Parse input, update DAG, return to `READY` | `Enter` / `Tab` / arrows per FR-023 | Click another cell (not pointing) | `N/A` | Spec 002 | `[x]` |
| `cell.cancelEdit` | Discard uncommitted input buffer | `Esc` | `N/A` | `N/A` | `N/A` | `[x]` |
| `point.setReference` | Insert or update live ref in formula/`DIALOG` | Arrows / `Shift+Arrows` / `F8` | Click or drag cells | `N/A` | `N/A` | `[x]` |

---

## 3. Range, History, File & View (`range.*`, `edit.*`, `file.*`, `view.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `range.erase` | Clear values/formulas in selection | `Delete` | Click `/Range/Erase` | `/Range/Erase` | Spec 002 | `[x]` |
| `range.copy` | Copy selection (with tiling & ref shift) | `/ R C` + prompt keys/arrows | Click `/Range/Copy` + grid drag | `/Range/Copy` | Spec 002 | `[x]` |
| `range.move` | Move selection (updating dependents) | `/ R M` + prompt keys/arrows | Click `/Range/Move` + grid drag | `/Range/Move` | Spec 002 | `[x]` |
| `edit.undo` | Revert last `proto123-ops` delta | `Ctrl+Z` | Click `/Edit/Undo` | `/Edit/Undo` | Spec 002 | `[x]` |
| `edit.redo` | Reapply undone `proto123-ops` delta | `Ctrl+Y` | Click `/Edit/Redo` | `/Edit/Redo` | Spec 002 | `[x]` |
| `file.newWorkbook` | Reset workbook (confirm if non-empty) | `/ F N` | Click `/File/New` + `[Yes]`/`[No]` | `/File/New` | Spec 002 | `[x]` |
| `view.cycleTheme` | Cycle built-in themes (`high-contrast`, `console16`, `mono`) | `/ V T` | Click `/View/Theme` | `/View/Theme` | `N/A` | `[x]` |

---

## 4. Menu, Prompt & Session Plumbing (`menu.*`, `prompt.*`, `session.*`)

| CommandId | Description | Keyboard | Mouse | `/` Menu | Headless CLI | Status |
|---|---|---|---|---|---|---|
| `menu.open` | Open Slash Command Bar at root | `/` | Click root-bar item | `N/A` | `N/A` | `[x]` |
| `menu.openAt` | Open specific root submenu | `N/A` | Click root-bar item | `N/A` | `N/A` | `[x]` |
| `menu.move` | Move highlight left/right (wraps) | `Left` / `Right` | `N/A` | `N/A` | `N/A` | `[x]` |
| `menu.hover` | Update hovered menu item | `N/A` | Pointer move over item | `N/A` | `N/A` | `[x]` |
| `menu.activate` | Descend into submenu or execute leaf | `Enter` | Click menu item | `N/A` | `N/A` | `[x]` |
| `menu.trigger` | Activate item by unique first letter | Letter key in `MENU` | `N/A` | `N/A` | `N/A` | `[x]` |
| `menu.back` | Step up one level or exit to `READY` | `Esc` | `N/A` | `N/A` | `N/A` | `[x]` |
| `menu.close` | Dismiss menu without changes | `Esc` at root | Click outside menu | `N/A` | `N/A` | `[x]` |
| `prompt.input` | Type into active prompt field | Printable key in `DIALOG` | `N/A` | `N/A` | `N/A` | `[x]` |
| `prompt.caret` | Move caret / edit prompt text | `Left`/`Right`/`Home`/`End`/`Backspace`/`Delete` | `N/A` | `N/A` | `N/A` | `[x]` |
| `prompt.submit` | Submit active dialog prompt | `Enter` | Click `[OK]` / `[Yes]` | `N/A` | `N/A` | `[x]` |
| `prompt.cancel` | Cancel active dialog prompt | `Esc` | Click `[Cancel]` / `[No]` | `N/A` | `N/A` | `[x]` |
| `prompt.focus` | Focus a dialog button | `Tab` / `Left` / `Right` | Hover button | `N/A` | `N/A` | `[x]` |
| `session.resize` | Update viewport grid size / report metrics mismatch | Terminal / browser resize | `N/A` | `N/A` | `N/A` | `[x]` |
| `session.setTheme` | Apply startup `--theme` / `--glyphs` override | CLI flags (`proto123-term`) | `N/A` | `N/A` | `N/A` | `[x]` |
