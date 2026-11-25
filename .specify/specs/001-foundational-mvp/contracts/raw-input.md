# Contract: Raw Input Events (both hosts → `proto123-tui::InputMapper`)

Hosts translate platform events into `RawEvent` **without interpreting them** (FR-030, FR-037, FR-045). All gesture logic — click vs. double-click, drag start/extend/end, hover changes, wheel step size, auto-scroll — lives in `InputMapper`.

```rust
pub struct RawEvent { pub t_ms: u64, pub kind: RawKind }   // t_ms: monotonic milliseconds from the host

pub enum RawKind {
    Key { key: Key, mods: Mods },                      // key press (no release needed)
    PointerDown { at: Pos, button: Button, mods: Mods },
    PointerMove { at: Pos, buttons_down: bool, mods: Mods },
    PointerUp   { at: Pos, button: Button, mods: Mods },
    Wheel { dx: i32, dy: i32, mods: Mods },            // notches; + = down/right
    Resize { cols: u16, rows: u16 },
    PointerLeave,
}
pub enum Pos { Cell { col: u16, row: u16 }, Pixel { x: f64, y: f64 } } // Pixel only from the web host
pub struct Mods { pub shift: bool, pub ctrl: bool, pub alt: bool, pub meta: bool }
pub enum Key { Char(char), Enter, Esc, Tab, BackTab, Backspace, Delete, Left, Right, Up, Down,
               Home, End, PageUp, PageDown, F(u8), Other }
```

## Web host mapping (`apps/proto123-wasm/src/translate.rs`, target-neutral)

| DOM | RawEvent |
|---|---|
| `keydown` `(key, code, shiftKey, ctrlKey, altKey, metaKey, timeStamp)` | `Key` — `key` strings `"ArrowLeft"`, `"F2"`, single chars, etc.; `Shift+Tab` → `BackTab` |
| `pointerdown/move/up` `(offsetX, offsetY, button, buttons, mods, timeStamp)` | `Pointer*` with `Pos::Pixel`; converted by `pixel_to_cell(.., CellMetrics)` before `InputMapper` |
| `wheel` `(deltaX, deltaY, deltaMode, mods)` | `Wheel` — sign of each delta → ±1 notch per event (`deltaMode` pixel/line/page all collapse to one notch), so 1 notch = 3 rows is decided in Rust |
| `ResizeObserver` / `devicePixelRatio` change | recompute grid `cols, rows` from canvas size and `CellMetrics` → `Resize`; full redraw |
| `pointerleave` | `PointerLeave` (clears hover) |

## Native host mapping (`apps/proto123-term/src/translate.rs`)

| crossterm | RawEvent |
|---|---|
| `Event::Key` (kind `Press` or `Repeat`) | `Key` (release ignored) |
| `MouseEventKind::Down/Up(btn)` | `PointerDown/Up` with `Pos::Cell` |
| `Drag(btn)` / `Moved` | `PointerMove { buttons_down: true/false }` |
| `ScrollUp/Down/Left/Right` | `Wheel` ±1 |
| `Event::Resize(c, r)` | `Resize` |

`t_ms` = `Instant::now() - start` in ms.

## Guarantees

- Keys the platform does not report are simply never produced; every command stays reachable via `F8`, plain keys, or the `/` menu (FR-024).
- Double-click: second `PointerDown` on the same `GridCell` within 500 ms (inclusive) of the first.
- Below 80×24 no `RawEvent` mutates the workbook; `Resize` back restores the layout.
