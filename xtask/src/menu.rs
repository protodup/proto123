use proto123_engine::menu::verify;
use proto123_engine::{CATALOG, REGISTRY};
use std::process::ExitCode;

pub fn run() -> ExitCode {
    let violations = verify(CATALOG, REGISTRY);
    if violations.is_empty() {
        println!("menu-commands: ok ({} commands)", REGISTRY.len());
        ExitCode::SUCCESS
    } else {
        for v in &violations {
            println!("menu-commands: {v}");
        }
        ExitCode::from(1)
    }
}
