//! Helpers of the architecture tests: workspace manifests and source scanning.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Dependency tables scanned at the top level and under every `[target.*]`.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// The workspace root, two levels above this package.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap_or_else(|| panic!("tests/architecture lives two levels below the workspace root"))
        .to_path_buf()
}

/// Reads and parses a TOML manifest.
pub fn read_manifest(path: &Path) -> toml::Table {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    text.parse::<toml::Table>()
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

/// One declared dependency of a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// The name it is declared under.
    pub key: String,
    /// The package it resolves to: `package = "..."` when renamed, otherwise the key.
    pub package: String,
    /// The `path = "..."` it points at, for workspace-internal dependencies.
    pub path: Option<String>,
    /// Whether it is declared as `workspace = true`.
    pub inherits_workspace: bool,
}

/// Every dependency of a manifest, including target-specific tables.
pub fn dependencies(manifest: &toml::Table) -> Vec<Dependency> {
    let mut found = Vec::new();
    collect_tables(manifest, &mut found);
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            collect_tables(target, &mut found);
        }
    }
    found
}

fn collect_tables(scope: &toml::Table, found: &mut Vec<Dependency>) {
    for name in DEPENDENCY_TABLES {
        let Some(entries) = scope.get(name).and_then(toml::Value::as_table) else {
            continue;
        };
        for (key, spec) in entries {
            let table = spec.as_table();
            let text = |field: &str| {
                table
                    .and_then(|table| table.get(field))
                    .and_then(toml::Value::as_str)
                    .map(str::to_owned)
            };
            found.push(Dependency {
                key: key.clone(),
                package: text("package").unwrap_or_else(|| key.clone()),
                path: text("path"),
                inherits_workspace: table
                    .and_then(|table| table.get("workspace"))
                    .and_then(toml::Value::as_bool)
                    == Some(true),
            });
        }
    }
}

/// A workspace package with its manifest.
#[derive(Debug)]
pub struct Member {
    /// `package.name`.
    pub name: String,
    /// The canonical directory of the package.
    pub dir: PathBuf,
    /// The parsed `Cargo.toml`.
    pub manifest: toml::Table,
    /// Every dependency the manifest declares.
    pub dependencies: Vec<Dependency>,
}

/// Every member listed in the root manifest. Globs are refused so the list stays explicit.
pub fn members() -> Vec<Member> {
    let root = workspace_root();
    let manifest = read_manifest(&root.join("Cargo.toml"));
    let listed = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| panic!("workspace.members must be an array"));
    listed
        .iter()
        .map(|entry| {
            let relative = entry
                .as_str()
                .unwrap_or_else(|| panic!("workspace member must be a string"));
            assert!(
                !relative.contains('*'),
                "list members explicitly: {relative}"
            );
            let dir = fs::canonicalize(root.join(relative))
                .unwrap_or_else(|error| panic!("member {relative}: {error}"));
            let manifest = read_manifest(&dir.join("Cargo.toml"));
            let name = manifest
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str)
                .unwrap_or_else(|| panic!("{relative} must define package.name"))
                .to_owned();
            let dependencies = dependencies(&manifest);
            Member {
                name,
                dir,
                manifest,
                dependencies,
            }
        })
        .collect()
}

/// The names under `[workspace.dependencies]`.
pub fn workspace_dependency_names() -> BTreeSet<String> {
    let manifest = read_manifest(&workspace_root().join("Cargo.toml"));
    manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

/// The workspace members `member` reaches through `path` dependencies, plus a description of
/// every path that does not lead to a member.
pub fn path_targets(member: &Member, all: &[Member]) -> (BTreeSet<String>, Vec<String>) {
    let mut targets = BTreeSet::new();
    let mut unresolved = Vec::new();
    for dependency in &member.dependencies {
        let Some(relative) = &dependency.path else {
            continue;
        };
        let target = fs::canonicalize(member.dir.join(relative))
            .ok()
            .and_then(|dir| all.iter().find(|candidate| candidate.dir == dir));
        match target {
            Some(target) => {
                targets.insert(target.name.clone());
            }
            None => unresolved.push(format!("{} = \"{relative}\"", dependency.key)),
        }
    }
    (targets, unresolved)
}

/// Every `.rs` file under `dir`, skipping build output and VCS metadata.
pub fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_sources(dir, &mut files);
    files.sort();
    files
}

fn collect_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if !matches!(entry.file_name().to_str(), Some("target" | ".git")) {
                collect_sources(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// `path` relative to the workspace root, with forward slashes.
pub fn relative_path(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Lines of `source` (1-based) that build a color from raw values, with what was found.
pub fn color_literal_violations(source: &str) -> Vec<(usize, String)> {
    let mut violations = Vec::new();
    for (index, raw_line) in source.lines().enumerate() {
        let line = strip_line_comment(raw_line);
        if let Some(literal) = find_hex_color(line) {
            violations.push((index + 1, format!("hex color literal `{literal}`")));
        }
        for constructor in ["rgb(", "rgba(", "hsl(", "hsla("] {
            if calls_function(line, constructor) {
                violations.push((index + 1, format!("color built with `{constructor}`")));
            }
        }
        if let Some(position) = line.find("Rgba {")
            && count_numeric_literals(&line[position..]) >= 3
        {
            violations.push((index + 1, "`Rgba {` built from numeric literals".to_owned()));
        }
    }
    violations
}

/// Whether `line` calls `constructor` (`rgb(`) as a function, not a longer name or a method.
fn calls_function(line: &str, constructor: &str) -> bool {
    line.match_indices(constructor).any(|(at, _)| {
        line[..at]
            .chars()
            .next_back()
            .is_none_or(|before| !(before.is_alphanumeric() || before == '_' || before == '.'))
    })
}

/// Drops a trailing `//` comment that sits outside a string.
fn strip_line_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    for index in 0..bytes.len().saturating_sub(1) {
        if bytes[index] == b'/' && bytes[index + 1] == b'/' {
            let quotes = bytes[..index].iter().filter(|byte| **byte == b'"').count();
            let after_space = index == 0 || bytes[index - 1].is_ascii_whitespace();
            if quotes % 2 == 0 && after_space {
                return &line[..index];
            }
        }
    }
    line
}

/// The first `#` followed by exactly 3, 6 or 8 hex digits.
fn find_hex_color(line: &str) -> Option<&str> {
    line.match_indices('#').find_map(|(at, _)| {
        let digits = line[at + 1..]
            .bytes()
            .take_while(u8::is_ascii_hexdigit)
            .count();
        matches!(digits, 3 | 6 | 8).then(|| &line[at..=at + digits])
    })
}

/// Counts numeric literals (integers or floats) in `text`.
fn count_numeric_literals(text: &str) -> usize {
    let mut count = 0;
    let mut in_number = false;
    for byte in text.bytes() {
        let is_number_byte = byte.is_ascii_digit() || (in_number && matches!(byte, b'.' | b'_'));
        if is_number_byte && !in_number {
            count += 1;
        }
        in_number = is_number_byte;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_specific_and_renamed_dependencies_are_collected() {
        let manifest: toml::Table = r#"
[dependencies]
plain = "1"
local = { path = "../local" }
shared = { workspace = true }

[target.'cfg(windows)'.dependencies]
renamed = { package = "real-name", version = "1" }

[target.'cfg(unix)'.dev-dependencies]
unix-only = "1"
"#
        .parse()
        .unwrap();
        let found = dependencies(&manifest);
        let find = |key: &str| {
            found
                .iter()
                .find(|dependency| dependency.key == key)
                .unwrap()
        };
        assert_eq!(find("renamed").package, "real-name");
        assert_eq!(find("unix-only").package, "unix-only");
        assert_eq!(find("local").path.as_deref(), Some("../local"));
        assert!(find("shared").inherits_workspace);
        assert!(!find("plain").inherits_workspace);
        assert_eq!(found.len(), 5);
    }

    #[test]
    fn color_detector_flags_constructors_and_ignores_methods() {
        let qualified = ["gpui", "::"].concat();
        for name in ["rgb", "rgba", "hsl", "hsla"] {
            let bare = format!("{name}(value)");
            let path = format!("{qualified}{name}(value)");
            assert_eq!(color_literal_violations(&bare).len(), 1, "{bare}");
            assert_eq!(color_literal_violations(&path).len(), 1, "{path}");
            for harmless in [
                format!("image.to_{name}(value)"),
                format!("other{name}(value)"),
                format!("let x = 1; // {bare}"),
            ] {
                assert!(color_literal_violations(&harmless).is_empty(), "{harmless}");
            }
        }
        assert_eq!(color_literal_violations("let c = \"#fff\";").len(), 1);
        assert_eq!(color_literal_violations("let c = \"#12345\";").len(), 0);
        assert_eq!(
            color_literal_violations("Rgba { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }").len(),
            1
        );
        assert_eq!(color_literal_violations("#[derive(Debug)]").len(), 0);
    }
}
