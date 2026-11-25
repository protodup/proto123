use cargo_metadata::{DependencyKind, MetadataCommand};
use std::collections::{HashMap, HashSet, VecDeque};
use std::process::ExitCode;
use xtask::layers::{Dep, Pkg, check};

const USAGE: &str = "usage: cargo xtask <command>\n\ncommands:\n  layers    check the L0-L6 dependency DAG (contracts/xtask.md)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("layers") => layers_command(),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
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
    let by_id: HashMap<_, _> = metadata.packages.iter().map(|p| (p.id.clone(), p)).collect();
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
            .map(|d| Dep { name: d.name.clone(), dev: d.kind == DependencyKind::Development })
            .collect();
        let transitive_normal = reachable_names(&package.id, &edges, &by_id);
        pkgs.push(Pkg { name: package.name.to_string(), deps, transitive_normal });
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
