//! Layer DAG check (research R12, contracts/xtask.md `layers`). Pure: takes a package model,
//! returns violation messages. `main.rs` builds the model from `cargo metadata`.

/// Where a workspace package sits in the architecture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// L0–L6, numbered.
    Numbered(u8),
    /// Test-only support crate; may depend on L0–L5, may only be a dev-dependency.
    Support,
    /// Repository tooling; nothing may depend on it.
    Tooling,
}

impl Layer {
    fn rank(self) -> u8 {
        match self {
            Self::Numbered(n) => n,
            Self::Support | Self::Tooling => 7,
        }
    }

    fn label(self) -> String {
        match self {
            Self::Numbered(n) => format!("L{n}"),
            Self::Support => "Support".to_owned(),
            Self::Tooling => "Tooling".to_owned(),
        }
    }
}

/// A registered package and its layer. Planned crates appear here too, so adding them later
/// only flips them from absent to checked (contracts/xtask.md).
pub const LAYERS: &[(&str, Layer)] = &[
    ("proto123-coord", Layer::Numbered(0)),
    ("proto123-value", Layer::Numbered(0)),
    ("proto123-theme", Layer::Numbered(0)),
    ("proto123-model", Layer::Numbered(1)),
    ("proto123-formula", Layer::Numbered(1)),
    ("proto123-calc", Layer::Numbered(2)),
    ("proto123-ops", Layer::Numbered(2)),
    ("proto123-ods", Layer::Numbered(3)),
    ("proto123-xlsx", Layer::Numbered(3)),
    ("proto123-csv", Layer::Numbered(3)),
    ("proto123-engine", Layer::Numbered(4)),
    ("proto123-tui", Layer::Numbered(5)),
    ("proto123-wasm", Layer::Numbered(6)),
    ("proto123-term", Layer::Numbered(6)),
    ("proto123-cli", Layer::Numbered(6)),
    ("proto123-testkit", Layer::Support),
    ("xtask", Layer::Tooling),
];

/// Third-party UI, terminal, and browser crates that must not appear at each layer.
const UI_LIBS_BELOW_L5: &[&str] = &[
    "ratatui",
    "crossterm",
    "ratzilla",
    "web-sys",
    "wasm-bindgen",
];
const UI_LIBS_IN_L5: &[&str] = &["crossterm", "ratzilla", "web-sys", "wasm-bindgen"];
/// Graphical-session and GPU crates that the native terminal app must never pull in (FR-044).
const GRAPHICAL_LIBS: &[&str] = &[
    "x11",
    "x11rb",
    "x11-clipboard",
    "wayland-client",
    "wgpu",
    "glutin",
];

/// One direct dependency of a workspace package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dep {
    pub name: String,
    /// True for `[dev-dependencies]`.
    pub dev: bool,
}

/// A workspace package as the checker sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pkg {
    pub name: String,
    pub deps: Vec<Dep>,
    /// Every package reachable through normal dependencies, transitively (for the graphical ban).
    pub transitive_normal: Vec<String>,
}

fn layer_of(name: &str) -> Option<Layer> {
    LAYERS.iter().find(|(n, _)| *n == name).map(|(_, l)| *l)
}

/// Returns one message per violation. An empty vector means the graph is clean.
pub fn check(pkgs: &[Pkg]) -> Vec<String> {
    let members: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    let mut out = Vec::new();
    for pkg in pkgs {
        let Some(layer) = layer_of(&pkg.name) else {
            out.push(format!(
                "{} is a workspace member but is not registered in LAYERS",
                pkg.name
            ));
            continue;
        };
        for dep in &pkg.deps {
            if dep.dev {
                continue;
            }
            let dep_is_member = members.contains(&dep.name.as_str());
            if dep_is_member {
                let Some(dep_layer) = layer_of(&dep.name) else {
                    out.push(format!(
                        "{} depends on unregistered member {}",
                        pkg.name, dep.name
                    ));
                    continue;
                };
                if dep_layer == Layer::Support {
                    out.push(format!(
                        "proto123-testkit must be a dev-dependency only, but {} depends on it normally",
                        pkg.name
                    ));
                } else if dep_layer == Layer::Tooling {
                    out.push(format!(
                        "{} depends on xtask (Tooling); tooling must not be a dependency",
                        pkg.name
                    ));
                } else if dep_layer.rank() > layer.rank() {
                    out.push(format!(
                        "{} ({}) -> {} ({})",
                        pkg.name,
                        layer.label(),
                        dep.name,
                        dep_layer.label()
                    ));
                }
            } else {
                let banned: &[&str] = match layer {
                    Layer::Numbered(5) => UI_LIBS_IN_L5,
                    Layer::Numbered(n) if n <= 4 => UI_LIBS_BELOW_L5,
                    _ => &[],
                };
                if banned.contains(&dep.name.as_str()) {
                    out.push(format!(
                        "{} ({}) depends on {}",
                        pkg.name,
                        layer.label(),
                        dep.name
                    ));
                }
            }
        }
        if pkg.name == "proto123-term" {
            for lib in GRAPHICAL_LIBS {
                if pkg.transitive_normal.iter().any(|n| n == lib) {
                    out.push(format!(
                        "proto123-term reaches graphical-session crate {lib} (FR-044)"
                    ));
                }
            }
        }
    }
    out
}
