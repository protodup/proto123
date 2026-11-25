#[test]
fn web_page_has_one_canvas_one_live_region_and_no_dom_creation_in_js() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let html_path = format!("{manifest_dir}/../../web/index.html");
    let js_path = format!("{manifest_dir}/../../web/host.js");

    let html = std::fs::read_to_string(&html_path).expect("read web/index.html");
    let js = std::fs::read_to_string(&js_path).expect("read web/host.js");

    assert_eq!(
        html.matches("<canvas").count(),
        1,
        "expected exactly one <canvas> in {html_path}"
    );
    assert_eq!(
        html.matches("aria-live").count(),
        1,
        "expected exactly one aria-live in {html_path}"
    );
    for forbidden in ["createElement", "appendChild", "innerHTML"] {
        assert!(
            !js.contains(forbidden),
            "forbidden DOM call {forbidden:?} found in {js_path}"
        );
    }
}
