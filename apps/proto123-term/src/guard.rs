//! Terminal state guard (research R4, FR-046): raw mode, alternate screen, cursor, and
//! mouse capture are restored on every exit path, including `Err` and panic.

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::{self, Write, stdout};
use std::sync::{Arc, Mutex};

/// Restores terminal modes on `out` in reverse startup order.
pub fn restore_to<W: Write>(out: &mut W, disable_raw: fn() -> io::Result<()>) {
    let _ = execute!(out, DisableMouseCapture, Show, LeaveAlternateScreen);
    let _ = disable_raw();
}

fn restore_stdout() {
    let mut out = stdout();
    restore_to(&mut out, disable_raw_mode);
}

pub struct TerminalGuard {
    active: bool,
    restorer: Box<dyn FnMut() + Send>,
}

impl TerminalGuard {
    /// Enters raw mode, alternate screen, hidden cursor, and SGR mouse capture on stdout.
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        if let Err(e) = execute!(out, EnterAlternateScreen, Hide, EnableMouseCapture) {
            restore_stdout();
            return Err(e);
        }
        Ok(Self {
            active: true,
            restorer: Box::new(restore_stdout),
        })
    }

    /// Testable entry point using an arbitrary writer and raw-mode callbacks.
    pub fn enter_with<W: Write + Send + 'static>(
        out: Arc<Mutex<W>>,
        enable_raw: fn() -> io::Result<()>,
        disable_raw: fn() -> io::Result<()>,
    ) -> io::Result<Self> {
        enable_raw()?;
        let write_result = {
            let mut guard = out
                .lock()
                .map_err(|_| io::Error::other("writer lock poisoned"))?;
            execute!(&mut *guard, EnterAlternateScreen, Hide, EnableMouseCapture)
        };
        if let Err(e) = write_result {
            if let Ok(mut guard) = out.lock() {
                restore_to(&mut *guard, disable_raw);
            } else {
                let _ = disable_raw();
            }
            return Err(e);
        }
        Ok(Self {
            active: true,
            restorer: Box::new(move || {
                if let Ok(mut guard) = out.lock() {
                    restore_to(&mut *guard, disable_raw);
                } else {
                    let _ = disable_raw();
                }
            }),
        })
    }

    /// Installs a panic hook that restores stdout before printing the panic message.
    pub fn install_panic_hook() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_stdout();
            previous(info);
        }));
    }

    /// Installs a panic hook that restores to `out` before printing the panic message.
    pub fn install_panic_hook_with<W: Write + Send + Sync + 'static>(
        out: Arc<Mutex<W>>,
        disable_raw: fn() -> io::Result<()>,
    ) {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if let Ok(mut guard) = out.lock() {
                restore_to(&mut *guard, disable_raw);
            } else {
                let _ = disable_raw();
            }
            previous(info);
        }));
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active {
            (self.restorer)();
            self.active = false;
        }
    }
}
