//! Raw, timestamped input events (contracts/raw-input.md). Hosts build these without
//! interpreting them.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawEvent {
    /// Monotonic milliseconds from the host.
    pub t_ms: u64,
    pub kind: RawKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawKind {
    Key {
        key: Key,
        mods: Mods,
    },
    PointerDown {
        at: Pos,
        button: Button,
        mods: Mods,
    },
    PointerMove {
        at: Pos,
        buttons_down: bool,
        mods: Mods,
    },
    PointerUp {
        at: Pos,
        button: Button,
        mods: Mods,
    },
    /// Notches; positive is down/right.
    Wheel {
        dx: i32,
        dy: i32,
        mods: Mods,
    },
    Resize {
        cols: u16,
        rows: u16,
    },
    PointerLeave,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pos {
    Cell {
        col: u16,
        row: u16,
    },
    /// Web host only: CSS pixels relative to the canvas.
    Pixel {
        x: f64,
        y: f64,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Other,
}
