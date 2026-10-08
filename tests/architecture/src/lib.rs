//! The layer policy of the workspace and the helpers the architecture tests share.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Dependency tables scanned at the top level and under every `[target.*]`.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// What one workspace package may depend on and contain. Adding a crate means adding a row.
#[derive(Debug)]
pub struct Layer {
    /// `package.name` in the crate's manifest.
    pub package: &'static str,
    /// Workspace crates it may depend on, as normal or dev dependency.
    pub workspace: &'static [&'static str],
    /// External crates it may depend on; `None` leaves them unrestricted, which marks the
    /// composition root.
    pub external: Option<&'static [&'static str]>,
    /// External crates it may use only as dev-dependencies, on top of `external`.
    pub external_dev: &'static [&'static str],
    /// Whether GPUI may appear in its manifest and sources.
    pub gpui_allowed: bool,
    /// Whether its sources may touch the file system, network, processes, threads or environment.
    pub io_allowed: bool,
    /// Whether it implements a port, so only the composition root's `main.rs` may name it.
    pub is_adapter: bool,
}

/// One row per workspace package.
pub const LAYERS: &[Layer] = &[
    Layer {
        package: "myapp-domain",
        workspace: &[],
        external: Some(&["thiserror"]),
        external_dev: &[],
        gpui_allowed: false,
        io_allowed: false,
        is_adapter: false,
    },
    Layer {
        package: "myapp-core",
        workspace: &["myapp-domain"],
        external: Some(&["thiserror", "tracing"]),
        external_dev: &[],
        gpui_allowed: false,
        io_allowed: false,
        is_adapter: false,
    },
    Layer {
        package: "myapp-storage",
        workspace: &["myapp-core"],
        external: Some(&["serde", "serde_json"]),
        external_dev: &["tempfile"],
        gpui_allowed: false,
        io_allowed: true,
        is_adapter: true,
    },
    Layer {
        package: "myapp-ui",
        workspace: &[],
        external: Some(&["gpui"]),
        external_dev: &[],
        gpui_allowed: true,
        io_allowed: true,
        is_adapter: false,
    },
    Layer {
        package: "architecture",
        workspace: &[],
        external: Some(&["toml"]),
        external_dev: &[],
        gpui_allowed: false,
        io_allowed: true,
        is_adapter: false,
    },
    Layer {
        package: "myapp",
        workspace: &["myapp-core", "myapp-storage", "myapp-ui"],
        external: None,
        external_dev: &[],
        gpui_allowed: true,
        io_allowed: true,
        is_adapter: false,
    },
];

impl Layer {
    /// The name the crate has in Rust code: the package name with `-` replaced by `_`.
    pub fn crate_name(&self) -> String {
        self.package.replace('-', "_")
    }
}

/// The policy row of a package; panics when the package is not registered.
pub fn layer(package: &str) -> &'static Layer {
    LAYERS
        .iter()
        .find(|layer| layer.package == package)
        .unwrap_or_else(|| {
            panic!("unknown workspace member `{package}`; register it in tests/architecture")
        })
}

/// The workspace root, two levels above this package.
pub fn workspace_root() -> &'static Path {
    static ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap_or_else(|| {
                panic!("tests/architecture lives two levels below the workspace root")
            });
        fs::canonicalize(root).unwrap_or_else(|error| panic!("workspace root: {error}"))
    });
    &ROOT
}

/// Reads a UTF-8 text file; panics with the path when it cannot be read.
pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// Reads and parses a TOML manifest.
pub fn read_manifest(path: &Path) -> toml::Table {
    read_text(path)
        .parse::<toml::Table>()
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

/// The parsed `Cargo.toml` of the workspace root.
pub fn root_manifest() -> toml::Table {
    read_manifest(&workspace_root().join("Cargo.toml"))
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
    /// Whether it is declared under `dev-dependencies`.
    pub dev: bool,
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
                dev: name == "dev-dependencies",
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

impl Member {
    /// The policy row of this member.
    pub fn layer(&self) -> &'static Layer {
        layer(&self.name)
    }
}

static MEMBERS: LazyLock<Vec<Member>> = LazyLock::new(|| {
    let root = workspace_root();
    let manifest = root_manifest();
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
});

/// Every member listed in the root manifest. Globs are refused so the list stays explicit.
pub fn members() -> &'static [Member] {
    &MEMBERS
}

/// The names under `[workspace.dependencies]`.
pub fn workspace_dependency_names() -> BTreeSet<String> {
    root_manifest()
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

/// A Rust source file of a workspace member.
#[derive(Debug)]
pub struct Source {
    /// Absolute path.
    pub path: PathBuf,
    /// Path relative to the workspace root, with forward slashes.
    pub relative: String,
    /// The file content.
    pub text: String,
}

static SOURCES: LazyLock<Vec<Source>> = LazyLock::new(|| {
    let mut paths = Vec::new();
    for member in members() {
        collect_sources(&member.dir, &mut paths);
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| Source {
            relative: relative_path(&path),
            text: read_text(&path),
            path,
        })
        .collect()
});

/// The `.rs` files under the directory of `member`, skipping build output and VCS metadata.
pub fn sources_of(member: &Member) -> impl Iterator<Item = &'static Source> {
    SOURCES
        .iter()
        .filter(|source| source.path.starts_with(&member.dir))
}

fn collect_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
        let path = entry.path();
        let kind = entry
            .file_type()
            .unwrap_or_else(|error| panic!("stat {}: {error}", path.display()));
        if kind.is_dir() {
            if !matches!(entry.file_name().to_str(), Some("target" | ".git")) {
                collect_sources(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn relative_path(path: &Path) -> String {
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
        assert!(find("unix-only").dev);
        assert!(!find("plain").dev);
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
