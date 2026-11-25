# Contract: Command Registry, Menu Tree, and Bindings (Spec 001)

Every state change goes through one of these `CommandId`s (FR-016). The same id is dispatched by the key, the mouse gesture, and the menu entry. `U` = creates one undo step. Names are the stable string ids used in docs, tests, and the parity checklist.

## Navigation & selection

| Name | Params | U | Keyboard | Mouse |
|---|---|---|---|---|
| `nav.moveRelative` | `dcol, drow` | – | Arrows, `Tab`/`Shift+Tab`, `Enter`/`Shift+Enter` (READY) | – |
| `nav.selectCell` | `addr` | – | (via moveRelative) | click cell |
| `nav.extendSelection` | `addr` | – | `Shift+Arrow`; arrows while `[EXT]` | `Shift+click`; drag |
| `nav.toggleExtendMode` | – | – | `F8` (READY, POINT); `Esc` ends | – (drag selects directly) |
| `nav.pageScroll` | `dir` | – | `PgUp`/`PgDn`; `Alt+PgUp`/`Alt+PgDn` horizontal | – |
| `nav.jumpHome` | `kind: RowStart \| A1` | – | `Home` / `Ctrl+Home` | – |
| `view.scroll` | `dcol, drow` | – | – | wheel = 3 rows/notch; `Shift+wheel` / horizontal = columns; drag past edge auto-scroll |

`Ctrl+Home` and `Alt+PgUp/PgDn` are convenience bindings; every target reaches the same cells with plain arrows/`PgUp`/`PgDn`/`Home` (FR-024).

## Entry & editing

| Name | Params | U | Keyboard | Mouse |
|---|---|---|---|---|
| `cell.beginEntry` | `first_char` | – | printable key in READY → VALUE/LABEL | – |
| `cell.beginEdit` | `addr?, caret?` | – | `F2` | double-click cell; click formula bar at char *n* |
| `cell.editInput` | `char` | – | printable in VALUE/LABEL/EDIT/POINT | – |
| `cell.editCaret` | `Left\|Right\|Home\|End\|Backspace\|Delete` | – | those keys in EDIT (and VALUE/LABEL buffer keys) | – |
| `cell.commitEdit` | `then: Move(dcol,drow) \| Select(addr) \| Stay` | ✔ | `Enter`, `Tab`, arrows per FR-023 | click another cell (not pointing) |
| `cell.cancelEdit` | – | – | `Esc` | – |
| `point.setReference` | `addr` / `range` | – | arrows in POINT (`F8` extends) | click / drag cell while pointing |

## Range, history, file, view

| Name | Params | U | Keyboard | Menu | Mouse |
|---|---|---|---|---|---|
| `range.erase` | `range` (selection) | ✔ | `Delete` | `/Range Erase` | click menu |
| `range.copy` | prompts `src`, `dest` | ✔ | – | `/Range Copy` | click menu; click/drag grid in prompt |
| `range.move` | prompts `src`, `dest` | ✔ | – | `/Range Move` | click menu; click/drag grid in prompt |
| `edit.undo` | – | – | `Ctrl+Z` | `/Edit Undo` | click menu |
| `edit.redo` | – | – | `Ctrl+Y` | `/Edit Redo` | click menu |
| `file.newWorkbook` | confirm if non-empty (default Cancel) | clears history | – | `/File New` | click menu, `[Yes]`/`[No]` |
| `view.cycleTheme` | – | – | – | `/View Theme` | click menu |

`Backspace` in READY is **not** bound to erase in this slice (interaction-spec §3 lists it; we keep only `Delete` to avoid accidental erase from a stray key — doc is updated to match).

## Menu, prompt, and session plumbing (not undoable, no menu entry)

`menu.open`, `menu.openAt(root_index)`, `menu.move(Left|Right)`, `menu.hover(index?)`, `menu.activate(index)`, `menu.trigger(letter)`, `menu.back`, `menu.close`, `prompt.input(char)`, `prompt.caret(..)`, `prompt.submit`, `prompt.cancel`, `prompt.focus(button)`, `session.resize(cols, rows)`, `session.setTheme(id, glyphs)` (startup override only).

Key/mouse: `/` and click on a root-bar item open the menu; `Left`/`Right` wrap; `Enter` or letter or click activates; `Esc` goes up; click outside closes. In prompts, `Enter`/`Esc`, typing, arrow-pointing, and grid click/drag fill the range text; hover highlights and click runs `[OK]`/`[Cancel]` (`[Yes]`/`[No]` for New).

## Menu tree (exact, FR-026)

```
File   (F)  → New    (N)  file.newWorkbook
Edit   (E)  → Undo   (U)  edit.undo
              Redo   (R)  edit.redo
Range  (R)  → Erase  (E)  range.erase
              Copy   (C)  range.copy
              Move   (M)  range.move
View   (V)  → Theme  (T)  view.cycleTheme
```

All labels and second-line descriptions come from `proto123_engine::strings` (FR-027). `cargo xtask menu-commands` verifies: every leaf id is in `REGISTRY`; triggers are unique per level and equal the label's first letter; every label/description key resolves.

## Outcome contract

`Session::dispatch` never panics and never reports success for something it did not do: Undo with empty history sets status `status.nothingToUndo`; an unparsable commit leaves the mode unchanged with status `diag.syntax` (`pos`, and `diag.useQuoteForText` when the entry did not start with `=`, `+`, `@`).
