# Contract: Web Host ↔ WASM Bridge (`web/` ↔ `apps/proto123-wasm`)

## Page shape (`web/index.html`) — FR-032, FR-036, SC-009

```html
<canvas id="grid" tabindex="0" aria-label="Proto123 spreadsheet"></canvas>
<div id="live" class="sr-only" aria-live="polite" aria-atomic="true"></div>
<!-- trunk: rel="rust" (apps/proto123-wasm, data-wasm-opt="z"), host.js, style.css -->
```

Exactly one drawing surface and one live region. No other elements are created at runtime. The canvas keeps focus; `host.js` calls `preventDefault()` on every `keydown` and `wheel` delivered to it while focused (so `Tab`, `/`, `F2`, `F8`, arrows reach the app), except browser-reserved combinations the browser never delivers.

## Exports (`#[wasm_bindgen]`, reached via `window.wasmBindings`)

| Export | Arguments | Returns |
|---|---|---|
| `start(canvas_id: &str)` | — | `()` — builds Ratzilla `CanvasBackend` on that canvas, initial `Resize`, first frame |
| `on_key(key: &str, code: &str, mods: u8, t_ms: f64)` | `mods` bits: 1 shift, 2 ctrl, 4 alt, 8 meta | `Option<String>` live-region text |
| `on_pointer(kind: u8, x: f64, y: f64, button: i16, buttons: u16, mods: u8, t_ms: f64)` | `kind`: 0 down, 1 move, 2 up, 3 leave; `x, y` = CSS px `offsetX/offsetY` | `Option<String>` |
| `on_wheel(dx: f64, dy: f64, mods: u8, t_ms: f64)` | raw deltas | `Option<String>` |
| `on_resize(css_w: f64, css_h: f64, dpr: f64)` | canvas client size and `devicePixelRatio` | `Option<String>` |

Each export: translate → `InputMapper` → `Session::dispatch` → `terminal.draw(...)` (Ratatui diff + Ratzilla dirty-cell paint) synchronously, then return the pending announcement (or `None`). The host writes a returned string into `#live` (`textContent`), nothing else.

## Announcement text (FR-036)

Built in Rust from the string catalog, emitted only on cell move, mode change, menu highlight change, or new status/error: `"<addr>, <raw or formula>, value <display>, <MODE>[, <status>]"`, e.g. `C4, formula =SUM(C1:C3), value 150, READY`; menu: `Menu Range, Copy: copy a range to another place, MENU`. Numbers use `format_full` (≤ 15 significant digits).

## Host size budget

`host.js` is plain JavaScript (no build step, no framework), target ≤ 200 lines: wait for `TrunkApplicationStarted`, call `start`, register listeners, forward primitives, write `#live`. No spreadsheet, layout, or gesture logic (FR-037).
