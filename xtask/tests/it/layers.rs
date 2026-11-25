use xtask::layers::{Dep, Pkg, check};

fn pkg(name: &str, deps: &[&str]) -> Pkg {
    Pkg {
        name: name.to_owned(),
        deps: deps
            .iter()
            .map(|d| Dep {
                name: (*d).to_owned(),
                dev: false,
            })
            .collect(),
        transitive_normal: Vec::new(),
    }
}

fn dev(name: &str, deps: &[&str]) -> Pkg {
    Pkg {
        name: name.to_owned(),
        deps: deps
            .iter()
            .map(|d| Dep {
                name: (*d).to_owned(),
                dev: true,
            })
            .collect(),
        transitive_normal: Vec::new(),
    }
}

fn clean_registry() -> Vec<Pkg> {
    vec![
        pkg("proto123-coord", &[]),
        pkg("proto123-value", &["proto123-coord"]),
        pkg("proto123-theme", &[]),
        pkg("proto123-formula", &["proto123-coord", "proto123-value"]),
        pkg(
            "proto123-model",
            &["proto123-coord", "proto123-value", "proto123-formula"],
        ),
        pkg("proto123-engine", &["proto123-model", "proto123-theme"]),
        pkg("proto123-tui", &["proto123-engine", "ratatui"]),
        pkg("proto123-testkit", &["proto123-tui"]),
        pkg(
            "proto123-wasm",
            &["proto123-tui", "ratzilla", "wasm-bindgen"],
        ),
        pkg("proto123-term", &["proto123-tui", "crossterm"]),
        pkg("xtask", &["proto123-engine"]),
    ]
}

#[test]
fn a_clean_graph_passes() {
    let mut pkgs = clean_registry();
    pkgs.push(dev("proto123-model", &["proto123-testkit"]));
    assert_eq!(check(&pkgs), Vec::<String>::new());
}

#[test]
fn upward_normal_dependency_is_reported_with_both_layers() {
    let mut pkgs = clean_registry();
    pkgs[0] = pkg("proto123-coord", &["proto123-engine"]);
    assert!(check(&pkgs).contains(&"proto123-coord (L0) -> proto123-engine (L4)".to_owned()));
}

#[test]
fn unregistered_member_is_reported() {
    let mut pkgs = clean_registry();
    pkgs.push(pkg("proto123-rogue", &[]));
    assert!(
        check(&pkgs)
            .iter()
            .any(|m| m.contains("proto123-rogue") && m.contains("not registered"))
    );
}

#[test]
fn testkit_as_normal_dependency_is_reported() {
    let mut pkgs = clean_registry();
    pkgs[0] = pkg("proto123-coord", &["proto123-testkit"]);
    assert!(
        check(&pkgs)
            .iter()
            .any(|m| m.contains("testkit") && m.contains("dev-dependency"))
    );
}

#[test]
fn ui_crates_are_banned_below_the_tui_layer() {
    for banned in [
        "ratatui",
        "crossterm",
        "ratzilla",
        "web-sys",
        "wasm-bindgen",
    ] {
        let mut pkgs = clean_registry();
        pkgs[1] = pkg("proto123-value", &["proto123-coord", banned]);
        assert!(
            check(&pkgs)
                .iter()
                .any(|m| m.contains(banned) && m.contains("L0")),
            "{banned} must be rejected in L0"
        );
    }
}

#[test]
fn backend_crates_are_banned_in_the_tui_layer() {
    let mut pkgs = clean_registry();
    pkgs[6] = pkg("proto123-tui", &["proto123-engine", "crossterm"]);
    assert!(
        check(&pkgs)
            .iter()
            .any(|m| m.contains("crossterm") && m.contains("L5"))
    );
}

#[test]
fn graphical_session_crates_are_banned_in_the_native_terminal() {
    let mut pkgs = clean_registry();
    let idx = pkgs
        .iter()
        .position(|p| p.name == "proto123-term")
        .unwrap_or(0);
    pkgs[idx].transitive_normal = vec!["crossterm".to_owned(), "wayland-client".to_owned()];
    assert!(
        check(&pkgs)
            .iter()
            .any(|m| m.contains("wayland-client") && m.contains("FR-044"))
    );
}

#[test]
fn same_layer_edges_are_allowed() {
    let mut pkgs = clean_registry();
    pkgs[3] = pkg("proto123-formula", &["proto123-coord", "proto123-value"]);
    pkgs[4] = pkg("proto123-model", &["proto123-formula", "proto123-coord"]);
    assert_eq!(check(&pkgs), Vec::<String>::new());
}
