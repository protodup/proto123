use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode};

const L0_TO_L5_CRATES: &[&str] = &[
    "proto123-coord",
    "proto123-value",
    "proto123-theme",
    "proto123-formula",
    "proto123-model",
    "proto123-calc",
    "proto123-ops",
    "proto123-engine",
    "proto123-tui",
];

/// Formats the baseline WASM size report line (contracts/xtask.md).
pub fn format_size_report(path: &str, raw: u64, gzip: u64) -> String {
    format!("wasm: {path} raw={raw} gzip={gzip}")
}

/// Compresses `bytes` with gzip level 9 and returns the compressed length.
pub fn gzip_size(bytes: &[u8]) -> Result<u64, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    let compressed = encoder.finish()?;
    Ok(compressed.len() as u64)
}

pub fn run() -> ExitCode {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--target", "wasm32-unknown-unknown"]);
    for crate_name in L0_TO_L5_CRATES {
        cmd.args(["-p", crate_name]);
    }
    match cmd.status() {
        Ok(st) if st.success() => {}
        Ok(_) => {
            eprintln!("wasm: L0-L5 wasm32-unknown-unknown build failed");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("wasm: failed to invoke cargo: {e}");
            return ExitCode::from(2);
        }
    }

    match Command::new("trunk").arg("--version").output() {
        Ok(out) if out.status.success() => {}
        _ => {
            eprintln!(
                "wasm: `trunk` not found; install with `cargo install --locked trunk --version 0.22.2`"
            );
            return ExitCode::from(2);
        }
    }

    match Command::new("trunk").args(["build", "--release"]).status() {
        Ok(st) if st.success() => {}
        Ok(_) => {
            eprintln!("wasm: `trunk build --release` failed");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("wasm: failed to invoke trunk: {e}");
            return ExitCode::from(2);
        }
    }

    let dist_dir = Path::new("dist");
    let entries = match std::fs::read_dir(dist_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("wasm: failed to read dist/: {e}");
            return ExitCode::from(1);
        }
    };
    let mut wasm_files = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().is_some_and(|ext| ext == "wasm") {
            wasm_files.push(p);
        }
    }
    wasm_files.sort();
    let Some(wasm_path) = wasm_files.first() else {
        eprintln!("wasm: no .wasm file produced in dist/");
        return ExitCode::from(1);
    };

    let bytes = match std::fs::read(wasm_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("wasm: failed to read {}: {e}", wasm_path.display());
            return ExitCode::from(1);
        }
    };
    let raw = bytes.len() as u64;
    let gzip = match gzip_size(&bytes) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("wasm: gzip compression failed: {e}");
            return ExitCode::from(1);
        }
    };
    let display_path = wasm_path.to_string_lossy().replace('\\', "/");
    println!("{}", format_size_report(&display_path, raw, gzip));
    ExitCode::SUCCESS
}
