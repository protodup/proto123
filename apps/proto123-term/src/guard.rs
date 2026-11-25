//! Terminal state guard (research R4, FR-046): raw mode and the alternate screen are restored
//! on every exit path, including a panic, because `Drop` runs and the panic hook restores first.

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use std::io::{self, Stdout, stdout};

pub struct TerminalGuard {
    active: bool,
}

/// Restores the terminal. Errors are ignored: this runs on the way out, and there is nothing
/// useful to do with a failed restore except try the remaining steps.
fn restore_terminal() {
    let mut out: Stdout = stdout();
    let _ = execute!(out, Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        if let Err(e) = execute!(out, EnterAlternateScreen, Hide) {
            restore_terminal();
            return Err(e);
        }
        Ok(Self { active: true })
    }

    /// Installs a panic hook that restores the terminal before the panic message prints.
    pub fn install_panic_hook() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous(info);
        }));
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active {
            restore_terminal();
            self.active = false;
        }
    }
}
