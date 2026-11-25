use xtask::docker::{find_wasm_href, parse_http_response};

#[test]
fn parses_http_status_line_and_lowercase_content_type() {
    let raw = "HTTP/1.1 200 OK\r\nServer: nginx\r\nContent-Type: application/wasm\r\nContent-Length: 4\r\n\r\n\0asm";
    let parsed = parse_http_response(raw).unwrap();
    assert_eq!(parsed.status, 200);
    assert_eq!(
        parsed.headers.get("content-type").map(String::as_str),
        Some("application/wasm")
    );
    assert_eq!(parsed.body, "\0asm");
}

#[test]
fn finds_wasm_href_in_trunk_index_html() {
    let html = r#"<!doctype html><html><head><link rel="preload" href="/proto123-wasm-deadbeef_bg.wasm" as="fetch"></head></html>"#;
    assert_eq!(
        find_wasm_href(html).as_deref(),
        Some("/proto123-wasm-deadbeef_bg.wasm")
    );
    assert_eq!(find_wasm_href("<html></html>"), None);
}
