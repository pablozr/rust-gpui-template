//! Rules about what the source files contain, per the layer policy table.

use architecture::{LAYERS, color_literal_violations, members, sources_of, std_io_uses};

const LINE_LIMIT: usize = 600;

/// The only file allowed to hold raw color values.
const TOKENS_FILE: &str = "crates/myapp-ui/src/tokens.rs";

#[test]
fn gpui_is_not_referenced_by_layers_that_forbid_it() {
    let needles = [
        concat!("use ", "gpui"),
        concat!("gpui", "::"),
        concat!("gpui_platform", "::"),
    ];
    let mut violations = Vec::new();
    for member in members()
        .iter()
        .filter(|member| !member.layer().gpui_allowed)
    {
        for source in sources_of(member) {
            for needle in needles {
                if source.text.contains(needle) {
                    violations.push(format!("{} references `{needle}`", source.relative));
                }
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn layers_that_forbid_io_do_none() {
    let tokens = ["extern \"C\"", "#[link"];
    let mut violations = Vec::new();
    for member in members().iter().filter(|member| !member.layer().io_allowed) {
        for source in sources_of(member) {
            for (line, module) in std_io_uses(&source.text) {
                violations.push(format!(
                    "{}:{line} uses `{module}`; I/O belongs behind a port",
                    source.relative
                ));
            }
            for (index, line) in source.text.lines().enumerate() {
                for token in tokens {
                    if line.contains(token) {
                        violations.push(format!(
                            "{}:{} uses `{token}`; I/O belongs behind a port",
                            source.relative,
                            index + 1
                        ));
                    }
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "{}",
        violations.join(
            "
"
        )
    );
}

#[test]
fn adapters_are_wired_only_in_the_composition_root_main() {
    let mut violations = Vec::new();
    for adapter in LAYERS.iter().filter(|layer| layer.is_adapter) {
        let crate_name = adapter.crate_name();
        for member in members()
            .iter()
            .filter(|member| member.name != adapter.package)
        {
            let main = member.dir.join("src").join("main.rs");
            let is_root = member.layer().external.is_none();
            for source in sources_of(member) {
                if is_root && source.path == main {
                    continue;
                }
                if source.text.contains(&crate_name) {
                    violations.push(format!(
                        "{} mentions adapter `{crate_name}`; only the composition root's \
                         main.rs may",
                        source.relative
                    ));
                }
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn color_literals_live_only_in_tokens() {
    let mut violations = Vec::new();
    for member in members()
        .iter()
        .filter(|member| member.layer().gpui_allowed)
    {
        for source in sources_of(member) {
            if source.relative == TOKENS_FILE {
                continue;
            }
            for (line, found) in color_literal_violations(&source.text) {
                violations.push(format!(
                    "{}:{line} has a {found}; colors belong in tokens.rs",
                    source.relative
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn no_mod_rs_files() {
    let found: Vec<&str> = members()
        .iter()
        .flat_map(sources_of)
        .filter(|source| source.path.file_name().is_some_and(|name| name == "mod.rs"))
        .map(|source| source.relative.as_str())
        .collect();
    assert!(
        found.is_empty(),
        "use name.rs next to name/ instead of: {found:?}"
    );
}

#[test]
fn source_files_stay_under_600_lines() {
    let mut violations = Vec::new();
    for source in members().iter().flat_map(sources_of) {
        let lines = source.text.lines().count();
        if lines > LINE_LIMIT {
            violations.push(format!(
                "{} has {lines} lines; split it by responsibility (limit {LINE_LIMIT})",
                source.relative
            ));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
