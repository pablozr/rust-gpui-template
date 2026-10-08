//! Rules about what the source files contain.

use std::fs;
use std::path::{Path, PathBuf};

use architecture::{color_literal_violations, relative_path, rust_sources, workspace_root};

const LINE_LIMIT: usize = 600;

/// Every Rust file of the app and the library crates, except this crate.
fn product_sources() -> Vec<PathBuf> {
    let root = workspace_root();
    ["apps", "crates"]
        .iter()
        .flat_map(|dir| rust_sources(&root.join(dir)))
        .collect()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn is_under(path: &Path, prefixes: &[&str]) -> bool {
    let relative = relative_path(path);
    prefixes.iter().any(|prefix| relative.starts_with(prefix))
}

#[test]
fn gpui_is_not_referenced_outside_the_app_and_the_ui_kit() {
    let needles = ["use gpui", "gpui::", "gpui_platform::"];
    let mut violations = Vec::new();
    for path in product_sources() {
        if is_under(&path, &["apps/myapp/", "crates/myapp-ui/"]) {
            continue;
        }
        let source = read(&path);
        for needle in needles {
            if source.contains(needle) {
                violations.push(format!("{} references `{needle}`", relative_path(&path)));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn domain_and_core_do_no_io() {
    let tokens = [
        "std::fs",
        "std::net",
        "std::process",
        "std::thread",
        "std::env",
        "extern \"C\"",
        "#[link",
    ];
    let mut violations = Vec::new();
    for path in product_sources() {
        if !is_under(
            &path,
            &["crates/myapp-domain/src/", "crates/myapp-core/src/"],
        ) {
            continue;
        }
        let source = read(&path);
        for (index, line) in source.lines().enumerate() {
            for token in tokens {
                if line.contains(token) {
                    violations.push(format!(
                        "{}:{} uses `{token}`; I/O belongs behind a port",
                        relative_path(&path),
                        index + 1
                    ));
                }
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn storage_is_wired_only_in_main() {
    let mut violations = Vec::new();
    for path in product_sources() {
        let relative = relative_path(&path);
        if relative == "apps/myapp/src/main.rs" || relative.starts_with("crates/myapp-storage/") {
            continue;
        }
        if read(&path).contains("myapp_storage") {
            violations.push(format!("{relative} mentions `myapp_storage`"));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn color_literals_live_only_in_tokens() {
    let mut violations = Vec::new();
    for path in product_sources() {
        let relative = relative_path(&path);
        if relative == "crates/myapp-ui/src/tokens.rs" {
            continue;
        }
        for (line, found) in color_literal_violations(&read(&path)) {
            violations.push(format!(
                "{relative}:{line} has a {found}; colors belong in tokens.rs"
            ));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn no_mod_rs_files() {
    let root = workspace_root();
    let found: Vec<String> = ["apps", "crates", "tests"]
        .iter()
        .flat_map(|dir| rust_sources(&root.join(dir)))
        .filter(|path| path.file_name().is_some_and(|name| name == "mod.rs"))
        .map(|path| relative_path(&path))
        .collect();
    assert!(
        found.is_empty(),
        "use name.rs next to name/ instead of: {found:?}"
    );
}

#[test]
fn source_files_stay_under_600_lines() {
    let root = workspace_root();
    let mut violations = Vec::new();
    for dir in ["apps", "crates", "tests"] {
        for path in rust_sources(&root.join(dir)) {
            let lines = read(&path).lines().count();
            if lines > LINE_LIMIT {
                violations.push(format!(
                    "{} has {lines} lines; split it by responsibility (limit {LINE_LIMIT})",
                    relative_path(&path)
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
