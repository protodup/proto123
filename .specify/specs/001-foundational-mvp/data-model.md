# Data Model: Foundational MVP (Spec 001)

Entities from the spec's Key Entities, placed in their owning crate. Field types are indicative; names are the contract for tasks and tests.

---

## L0 — `proto123-coord`

### CellAddr
| Field | Type | Rule |
|---|---|---|
| `col` | `u32` | 0-based, `0..=16_383` (`A`..`XFD`) |
| `row` | `u32` | 0-based, `0..=1_048_575` (`1`..`1048576`) |

- Construction is fallible (`CellAddr::new(col,row) -> Option<Self>`); no type can hold an out-of-grid address.
- `Display` → `A1` (upper-case letters). `Hash + Ord` (row-major) for sparse keys.

### RefAddr (a reference as written in a formula)
`{ addr: CellAddr, col_abs: bool, row_abs: bool }` — parse/print `A1`, `$A1`, `A$1`, `$A$1`; letters case-insensitive on input (FR-002).
- `shifted(dc: i64, dr: i64) -> Option<RefAddr>`: absolute parts unchanged; `None` if the result leaves the grid (→ `#REF!`).

### CellRange
`{ start: CellAddr, end: CellAddr }`, always normalized so `start ≤ end` on both axes. `contains`, `width`, `height`, `area: u64`, `iter()` (row-major). Parse `A1:B3` in any corner order.

---

## L0 — `proto123-value`

### Value
`Empty | Number(f64) | Text(String) | Bool(bool) | Error(ErrorKind)`
- Invariant: `Number` is always finite; arithmetic producing ±∞/NaN yields `Error(Value)` (or `Error(Div0)` for x/0).

### ErrorKind
`Div0 | Ref | Name | Value | Circ` → tokens `#DIV/0!`, `#REF!`, `#NAME?`, `#VALUE!`, `#CIRC!`.

### Display
`format_general(n: f64, width: usize) -> String` (FR-006, steps 1–4); `format_full(n)` = 15-significant-digit form used by the formula bar and live region. Text is shown as-is; Bool as `TRUE`/`FALSE`.

---

## L0 — `proto123-theme`

| Entity | Fields |
|---|---|
| `Color` | `Rgb(u8,u8,u8)` \| `Ansi16(Ansi16)` |
| `GlyphSet` | `Box` (single-line `─│┌┐└┘├┤┬┴┼`) \| `Ascii` (`-|+`) |
| `ThemeTokens` | named `(fg, bg)` pairs: `grid`, `header`, `active_cell`, `selection`, `menu`, `menu_highlight`, `control_line`, `status`, `badge`, `error`, `border` |
| `Theme` | `{ id: ThemeId, tokens: ThemeTokens, glyphs: GlyphSet, allow_bold: bool, metrics: GridMetrics }` |
| `GridMetrics` | character-cell layout metrics: `col_width: u16` (= 9, FR-006), `min_cols: u16` (= 80), `min_rows: u16` (= 24); identical in every built-in theme in this slice |
| `ThemeId` | `HighContrast` \| `Console16` \| `Mono` (cycle order) |
| `CapabilityProfile` | `Full` \| `Ansi16` \| `Console` \| `Conservative`; `detect(term, colorterm)`; `default_theme()`, `default_glyphs()` |

Invariants: `Console16` uses only `Ansi16` colors and `allow_bold = false`; every theme passes `contrast(fg,bg) ≥ 4.5` for text tokens and `≥ 3.0` for `border`/`active_cell`/`menu_highlight`.

---

## L1 — `proto123-formula`

### Expr (AST)
```
Expr = Number(f64) | Bool(bool) | ErrorLit(ErrorKind)
     | Ref(RefAddr) | Range(RefAddr, RefAddr)
     | Unary(UnOp, Box<Expr>) | Binary(BinOp, Box<Expr>, Box<Expr>)
     | Call { name: String /* upper-cased */, args: Vec<Expr> }
UnOp = Neg | Plus        BinOp = Add | Sub | Mul | Div | Pow
```
- `Formula { expr: Expr, canonical: String }` — `canonical` always starts with `=` (FR-012) and re-parses to an equal `Expr`.
- `references() -> (Vec<CellAddr>, Vec<CellRange>)` feeds the dependency graph.
- `shifted(dc, dr) -> Expr` (copy) — out-of-grid refs become `ErrorLit(Ref)`.
- `remap(&dyn Fn(RefAddr) -> RefRemap) -> Expr` (move) — `Keep | To(RefAddr) | Invalid`.
- `ParseError { pos: usize /* char index */, kind: ParseErrorKind }`; kinds include `Unexpected`, `UnclosedParen`, `TooLong` (> 8,192 chars), `TooDeep` (> 256 nesting).

### CellSource (trait implemented in L2)
`value(addr) -> Value`; `for_each_in_range(range, f)` visits only populated cells.

### Evaluation rules
Empty→0, Bool→1/0, Text→`#VALUE!`, first error operand wins (left to right), aggregates skip Text/Empty/Bool in ranges but propagate errors; `AVERAGE` of no numbers → `#DIV/0!`; `MIN`/`MAX`/`SUM` → 0; `COUNT` counts numbers. Direct (non-range) arguments to aggregates: numbers and Booleans count as numbers, Text → `#VALUE!`. Unknown name → `#NAME?`.

---

## L1 — `proto123-model`

### Alignment
`Left | Right | Center` — from label prefix `'` / `"` / `^`; unprefixed labels Left; numbers Right.

### CellContent (raw input)
`Number(f64) | Label { text: String, align: Alignment, prefix: Option<char> } | Formula(Formula)`
- The formula bar shows `prefix + text` for labels, `format_full(n)` for numbers, `canonical` for formulas.

### Cell
`{ content: CellContent, value: Value /* cached result */ }`. An absent map entry is Empty; storing Empty removes the entry (sparsity invariant, FR-001).

### Sheet / Workbook
`Sheet { cells: HashMap<CellAddr, Cell> }`; `Workbook { sheet: Sheet }` (exactly one sheet in this slice). `is_empty()` drives the File New confirmation.

---

## L2 — `proto123-calc`

### DependencyGraph
| Field | Meaning |
|---|---|
| `precedents: HashMap<CellAddr, Precedents { cells, ranges }>` | what each formula reads |
| `cell_dependents: HashMap<CellAddr, HashSet<CellAddr>>` | reverse single-cell edges |
| `range_dependents: Vec<(CellRange, CellAddr)>` | reverse range edges (linear scan) |

### CalcSheet
`{ workbook: Workbook, graph: DependencyGraph }` — the only writer of cell contents.
- `apply(changes: &[(CellAddr, Option<CellContent>)]) -> RecalcReport` updates contents + graph, then recalculates the dirty subgraph once.
- `RecalcReport { evaluated: Vec<CellAddr>, changed_display: Vec<CellAddr> }` — `evaluated.len()` is the SC-003 count; `changed_display` drives redraw.
- Invariants after every `apply`: every formula's cached `value` equals a fresh evaluation; cycle members and their dependents hold `Error(Circ)`; no recursion depth proportional to chain length.

---

## L2 — `proto123-ops`

| Entity | Fields / behavior |
|---|---|
| `CellDelta` | `changes: Vec<(CellAddr, Option<CellContent> /*before*/, Option<CellContent> /*after*/)>`; `invert()` |
| `Op` | `SetCell { addr, content }`, `Erase { range }`, `Copy { src, dest }`, `Move { src, dest }` → each `fn plan(&CalcSheet) -> CellDelta` (pure), then `CalcSheet::apply(after-side)` |
| `UndoStack` | `undo: VecDeque<CellDelta>`, `redo: Vec<CellDelta>`, capacity 1,000; `push` clears `redo`; `clear()` on File New |

Copy tiling and Move rewrite rules: research R10.

---

## L4 — `proto123-engine`

### Mode
`Ready | Value | Label | Edit | Point | Menu | Dialog` — badge text via the string catalog (FR-017).

### Session
| Field | Type | Notes |
|---|---|---|
| `doc` | `CalcSheet` | |
| `undo` | `UndoStack` | |
| `mode` | `Mode` | exactly one |
| `active` | `CellAddr` | |
| `anchor` | `Option<CellAddr>` | selection = `CellRange(anchor, active)` when set |
| `extend_mode` | `bool` | `F8`; shows `[EXT]` |
| `edit` | `Option<EditState { buffer: String, caret: usize /* char index */, origin_mode, point: Option<PointState> }>` | present in Value/Label/Edit/Point; `PointState { insert_at: usize, len: usize, anchor: Option<CellAddr>, cursor: CellAddr }` is the live reference span in the buffer, `Some` only in Point |
| `menu` | `Option<MenuState { path: Vec<usize>, highlighted: usize, hovered: Option<usize> }>` | |
| `prompt` | `Option<Prompt>` | `ConfirmNew { focus: Yes/No }`, `RangeSource { cmd, text }`, `RangeDest { cmd, src, text }` |
| `viewport` | `Viewport { top: u32, left: u32 }` | scroll origin; visible rows/cols come from layout |
| `theme` | `ThemeId` + `GlyphSet` override | |
| `status` | `Option<Status { key: StringKey, args, kind: Info/Error }>` | |
| `announce` | `Option<String>` | pending live-region text, built only by the engine (`announce.rs`) and returned as `Outcome.announce`; drained by the web host |

### Command
`CommandId` enum + `CommandSpec { id, name, undoable }`; full list in [contracts/commands.md](contracts/commands.md). `Session::dispatch(Command) -> Outcome { redraw: Redraw::{None, Regions, Full}, announce: Option<String> }`.

### MenuNode
`{ label: StringKey, desc: StringKey, trigger: char, kind: Branch(&'static [MenuNode]) | Leaf(CommandId) }`.

### Mode transitions (FR-017; interaction-spec §2 as amended by this spec)
```
READY --digit = + - . ( @--> VALUE        READY --other printable, ' " ^--> LABEL
READY --F2 / dbl-click / formula-bar click--> EDIT
VALUE|EDIT --arrow/click/drag when caret after operator ( , ; : --> POINT
POINT --typed operator/char--> back to VALUE|EDIT (reference kept)
VALUE|LABEL --arrow (not pointing)--> commit + move --> READY
EDIT --Up/Down--> commit + move --> READY;  EDIT --Left/Right/Home/End--> caret only
VALUE|LABEL|EDIT|POINT --Enter/Tab/click elsewhere--> commit --> READY  (parse error: stay, diagnostic)
VALUE|LABEL|EDIT|POINT --Esc--> discard --> READY
READY --/ or root-bar click--> MENU;  MENU --Esc at root / click outside--> READY; Esc in submenu --> parent
MENU --leaf needing input (Copy, Move, New with content)--> DIALOG
DIALOG --Enter/OK--> run command --> READY;  DIALOG --Esc/Cancel--> READY (no change)
```

---

## L5 — `proto123-tui`

| Entity | Fields |
|---|---|
| `CellMetrics` | `cell_w: f64, cell_h: f64, origin_x: f64, origin_y: f64, dpr: f64` (CSS px). Web only: the pixel size of one character cell, measured from the backend's font; grid layout metrics come from `Theme.metrics` |
| `pixel_to_cell(x, y, &CellMetrics, cols, rows) -> Option<(u16,u16)>` | pure; `None` outside the grid |
| `HitTarget` | `GridCell(CellAddr)`, `ColHeader(u32)`, `RowHeader(u32)`, `FormulaBar { char_index }`, `AddressBox`, `MenuItem { level, index }`, `PromptField { char_index }`, `PromptButton(Ok/Cancel/Yes/No)`, `GridEdge(Up/Down/Left/Right)`, `None` |
| `LayoutMap` | per frame: list of `(Rect, HitTarget-producer)`; `hit(col,row) -> HitTarget` |
| `RawEvent` | see [contracts/raw-input.md](contracts/raw-input.md) |
| `InputMapper` | `{ last_press: Option<(HitTarget, t_ms)>, drag: Option<DragState>, hover: Option<HitTarget> }`; `map(RawEvent, &LayoutMap, &Session) -> Vec<Command>`; double-click = 2nd press on same `GridCell` within 500 ms |
| `Frame` | `render(&Session, area, &mut Buffer) -> LayoutMap` (announcement text comes from the engine, `Session.announce`, not from rendering); below `GridMetrics.min_cols`×`min_rows` (80×24) → "enlarge the window" frame with an empty `LayoutMap` (no mutating hit targets) |

---

## Relationships

```
Workbook 1─1 Sheet 1─* Cell ─ CellContent ─ Formula ─ Expr ─* RefAddr/CellRange
CalcSheet ─ Workbook + DependencyGraph            UndoStack ─* CellDelta
Session ─ CalcSheet, UndoStack, Mode, MenuState, Prompt, Viewport, Theme
LayoutMap ←(same pass)─ render(Session) ─→ Buffer      InputMapper(RawEvent, LayoutMap) ─→ Command ─→ Session::dispatch
```
