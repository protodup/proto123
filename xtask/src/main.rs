use cargo_metadata::{DependencyKind, MetadataCommand};
use std::collections::{HashMap, HashSet, VecDeque};
use std::process::{Command, ExitCode};
use xtask::layers::{Dep, Pkg, check};

const USAGE: &str = "usage: cargo xtask <command>

commands:
  layers         check the L0-L6 dependency DAG (contracts/xtask.md)
  menu-commands  verify every menu entry maps to a live CommandId
  wasm           build L0-L5 + trunk release bundle and report .wasm size
  docker-check   build container image and verify WASM Content-Type
  ci             run fmt, clippy, layers, menu-commands, tests, and wasm";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("layers") => layers_command(),
        Some("menu-commands") => xtask::menu::run(),
        Some("wasm") => xtask::wasm::run(),
        Some("docker-check") => xtask::docker::run(),
        Some("ci") => ci_command(),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run_step(label: &str, mut cmd: Command) -> Result<(), ExitCode> {
    println!("ci: running {label}");
    match cmd.status() {
        Ok(st) if st.success() => Ok(()),
        Ok(st) => {
            eprintln!("ci: step `{label}` failed");
            let code = u8::try_from(st.code().unwrap_or(1)).unwrap_or(1);
            Err(ExitCode::from(code.max(1)))
        }
        Err(e) => {
            eprintln!("ci: step `{label}` could not start: {e}");
            Err(ExitCode::from(2))
        }
    }
}

fn ci_command() -> ExitCode {
    let mut fmt = Command::new("cargo");
    fmt.args(["fmt", "--all", "--", "--check"]);
    if let Err(code) = run_step("fmt", fmt) {
        return code;
    }

    let mut clippy = Command::new("cargo");
    clippy.args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    if let Err(code) = run_step("clippy", clippy) {
        return code;
    }

    println!("ci: running layers");
    let layers_code = layers_command();
    if layers_code != ExitCode::SUCCESS {
        eprintln!("ci: step `layers` failed");
        return layers_code;
    }

    println!("ci: running menu-commands");
    let menu_code = xtask::menu::run();
    if menu_code != ExitCode::SUCCESS {
        eprintln!("ci: step `menu-commands` failed");
        return menu_code;
    }

    let mut test = Command::new("cargo");
    test.args(["test", "--workspace"]);
    if let Err(code) = run_step("test", test) {
        return code;
    }

    println!("ci: running wasm");
    let wasm_code = xtask::wasm::run();
    if wasm_code != ExitCode::SUCCESS {
        eprintln!("ci: step `wasm` failed");
        return wasm_code;
    }

    println!("ci: ok");
    ExitCode::SUCCESS
}

fn layers_command() -> ExitCode {
    let metadata = match MetadataCommand::new().exec() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("layers: cargo metadata failed: {e}");
            return ExitCode::from(2);
        }
    };
    let members: HashSet<_> = metadata.workspace_members.iter().cloned().collect();
    let by_id: HashMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.clone(), p))
        .collect();
    let edges: HashMap<_, Vec<_>> = metadata
        .resolve
        .as_ref()
        .map(|r| {
            r.nodes
                .iter()
                .map(|n| {
                    let normal: Vec<_> = n
                        .deps
                        .iter()
                        .filter(|d| d.dep_kinds.iter().any(|k| k.kind == DependencyKind::Normal))
                        .map(|d| d.pkg.clone())
                        .collect();
                    (n.id.clone(), normal)
                })
                .collect()
        })
        .unwrap_or_default();

    let mut pkgs = Vec::new();
    for package in metadata.packages.iter().filter(|p| members.contains(&p.id)) {
        let deps = package
            .dependencies
            .iter()
            .map(|d| Dep {
                name: d.name.clone(),
                dev: d.kind == DependencyKind::Development,
            })
            .collect();
        let transitive_normal = reachable_names(&package.id, &edges, &by_id);
        pkgs.push(Pkg {
            name: package.name.to_string(),
            deps,
            transitive_normal,
        });
    }

    let violations = check(&pkgs);
    if violations.is_empty() {
        println!("layers: ok ({} workspace packages)", pkgs.len());
        ExitCode::SUCCESS
    } else {
        for v in &violations {
            println!("layers: {v}");
        }
        ExitCode::from(1)
    }
}

/// Names of every package reachable from `start` through normal dependencies.
fn reachable_names(
    start: &cargo_metadata::PackageId,
    edges: &HashMap<cargo_metadata::PackageId, Vec<cargo_metadata::PackageId>>,
    by_id: &HashMap<cargo_metadata::PackageId, &cargo_metadata::Package>,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([start.clone()]);
    let mut names = Vec::new();
    while let Some(id) = queue.pop_front() {
        for next in edges.get(&id).into_iter().flatten() {
            if seen.insert(next.clone()) {
                if let Some(p) = by_id.get(next) {
                    names.push(p.name.to_string());
                }
                queue.push_back(next.clone());
            }
        }
    }
    names
}
