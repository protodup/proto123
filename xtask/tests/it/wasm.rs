use xtask::wasm::{format_size_report, gzip_size};

#[test]
fn formats_wasm_size_report_line() {
    assert_eq!(
        format_size_report("dist/proto123-wasm-abc123_bg.wasm", 123456, 45678),
        "wasm: dist/proto123-wasm-abc123_bg.wasm raw=123456 gzip=45678"
    );
}

#[test]
fn gzip_size_compresses_repeating_bytes() {
    let raw = vec![0u8; 4096];
    let gz = gzip_size(&raw).unwrap();
    assert!(gz > 0 && gz < raw.len() as u64);
}
