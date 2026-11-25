# Contract: `proto123-term` Command Line and Terminal Lifecycle

## Invocation

```text
proto123-term [--theme high-contrast|console16|mono] [--glyphs box|ascii] [--help] [--version]
```

- No positional file/URL argument in this slice (file I/O is Spec 002). A positional argument is rejected with a clear message on stderr and exit code 2 — never silently ignored (FR-043).
- `--theme` / `--glyphs` override the detected `CapabilityProfile` (FR-047, US8 sc.4). Unknown values → message + exit 2.
- Arguments are parsed by hand from `std::env::args` (no argument-parser dependency).

## Startup

1. Read `TERM`, `COLORTERM`; `CapabilityProfile::detect(...)` (core) → theme + glyphs; apply overrides.
2. Install panic hook, create `TerminalGuard`: raw mode, alternate screen, hide cursor, `EnableMouseCapture`.
3. Initial `Resize` from `crossterm::terminal::size()`; draw.

## Loop

`crossterm::event::read()` → `translate` → `InputMapper` → `Session::dispatch` → `terminal.draw` (Ratatui diff writes only changed cells). Resize → full redraw. `Ctrl+Q` in `READY` quits.

> Why `Ctrl+Q` is enough: the spec's menu tree is exact (FR-026) and has no Quit entry. FR-024 targets combinations some environments do not report (`Shift+Arrow`, `Ctrl+PgUp`); `Ctrl+Q` is a plain control character (0x11) that every terminal, including the Linux console, delivers. Quit is a host action, not a `CommandId`: it changes no workbook or session state. The app does not disable XON/XOFF handling explicitly; crossterm raw mode already does (`IXON` off), so `Ctrl+Q` reaches the app.

## Exit

Normal quit, `Err` from the loop, or panic: the guard/hook disables mouse capture, leaves the alternate screen, disables raw mode, shows the cursor, **then** any error is printed to stderr. Exit codes: 0 normal, 1 internal error, 2 usage error (SC-014).
