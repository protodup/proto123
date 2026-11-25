use proto123_term::args::{CliAction, parse_args};
use proto123_term::guard::TerminalGuard;
use proto123_theme::{GlyphSet, ThemeId};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static RAW_ENABLED: AtomicBool = AtomicBool::new(false);

fn fake_enable_raw() -> io::Result<()> {
    RAW_ENABLED.store(true, Ordering::SeqCst);
    Ok(())
}

fn fake_disable_raw() -> io::Result<()> {
    RAW_ENABLED.store(false, Ordering::SeqCst);
    Ok(())
}

fn assert_restored_sequences(bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    // Enter sequence: alt screen, hide cursor, enable mouse capture (?1000h/?1002h/?1003h/?1006h).
    assert!(
        text.contains("\x1b[?1049h"),
        "missing EnterAlternateScreen: {text:?}"
    );
    assert!(text.contains("\x1b[?25l"), "missing Hide: {text:?}");
    assert!(
        text.contains("\x1b[?1000h"),
        "missing EnableMouseCapture: {text:?}"
    );
    // Restore sequence: disable mouse capture, show cursor, leave alternate screen.
    assert!(
        text.contains("\x1b[?1000l"),
        "missing DisableMouseCapture: {text:?}"
    );
    assert!(text.contains("\x1b[?25h"), "missing Show: {text:?}");
    assert!(
        text.contains("\x1b[?1049l"),
        "missing LeaveAlternateScreen: {text:?}"
    );
}

#[test]
fn guard_restores_on_normal_drop_and_err() {
    // Normal drop.
    let buf = Arc::new(Mutex::new(Vec::new()));
    {
        let _guard =
            TerminalGuard::enter_with(Arc::clone(&buf), fake_enable_raw, fake_disable_raw).unwrap();
        assert!(RAW_ENABLED.load(Ordering::SeqCst));
    }
    assert!(!RAW_ENABLED.load(Ordering::SeqCst));
    assert_restored_sequences(&buf.lock().unwrap());

    // Early return via Err.
    let buf2 = Arc::new(Mutex::new(Vec::new()));
    let res: io::Result<()> = (|| {
        let _guard =
            TerminalGuard::enter_with(Arc::clone(&buf2), fake_enable_raw, fake_disable_raw)?;
        Err(io::Error::other("simulated loop failure"))
    })();
    assert!(res.is_err());
    assert!(!RAW_ENABLED.load(Ordering::SeqCst));
    assert_restored_sequences(&buf2.lock().unwrap());
}

#[test]
fn guard_restores_on_panic() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let hook_buf = Arc::clone(&buf);
    let prev = std::panic::take_hook();
    TerminalGuard::install_panic_hook_with(hook_buf, fake_disable_raw);
    let result = std::panic::catch_unwind(|| {
        let _guard =
            TerminalGuard::enter_with(Arc::clone(&buf), fake_enable_raw, fake_disable_raw).unwrap();
        assert!(RAW_ENABLED.load(Ordering::SeqCst));
        panic!("simulated crash");
    });
    std::panic::set_hook(prev);
    assert!(result.is_err());
    assert!(!RAW_ENABLED.load(Ordering::SeqCst));
    assert_restored_sequences(&buf.lock().unwrap());
}

#[test]
fn cli_args_parse_overrides_help_version_and_reject_invalid() {
    assert_eq!(
        parse_args(["--theme", "console16", "--glyphs", "ascii"]),
        Ok(CliAction::Run {
            theme: Some(ThemeId::Console16),
            glyphs: Some(GlyphSet::Ascii),
        })
    );
    assert_eq!(parse_args(["--help"]), Ok(CliAction::Help));
    assert_eq!(parse_args(["--version"]), Ok(CliAction::Version));
    assert!(parse_args(["budget.ods"]).is_err());
    assert!(parse_args(["--theme", "unknown"]).is_err());
    assert!(parse_args(["--glyphs", "emoji"]).is_err());
    assert!(parse_args(["--bogus"]).is_err());
}
