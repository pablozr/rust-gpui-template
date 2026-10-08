//! Which crates may depend on which, according to the layer policy table.

use architecture::{Dependency, members, path_targets, workspace_dependency_names};

fn external_dependencies(dependencies: &[Dependency]) -> impl Iterator<Item = &Dependency> {
    dependencies
        .iter()
        .filter(|dependency| dependency.path.is_none())
}

fn is_gpui(package: &str) -> bool {
    package == "gpui" || package.starts_with("gpui_")
}

#[test]
fn every_member_is_registered_and_uses_only_allowed_workspace_crates() {
    let all = members();
    let mut violations = Vec::new();
    for member in all {
        let allowed = member.layer().workspace;
        let (targets, unresolved) = path_targets(member, all);
        for path in unresolved {
            violations.push(format!(
                "`{}` has path dependency {path} that is not a workspace member",
                member.name
            ));
        }
        for target in targets {
            if !allowed.contains(&target.as_str()) {
                violations.push(format!("`{}` must not depend on `{target}`", member.name));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn restricted_layers_use_only_allow_listed_external_crates() {
    let mut violations = Vec::new();
    for member in members() {
        let layer = member.layer();
        let Some(allowed) = layer.external else {
            continue;
        };
        for dependency in external_dependencies(&member.dependencies) {
            let package = dependency.package.as_str();
            let listed = allowed.contains(&package)
                || (dependency.dev && layer.external_dev.contains(&package));
            if !listed {
                let kind = if dependency.dev {
                    "dev-dependency"
                } else {
                    "dependency"
                };
                violations.push(format!(
                    "`{}` may not use `{package}` as a {kind}; allowed: {allowed:?}, \
                     dev-only: {:?}",
                    member.name, layer.external_dev
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn gpui_is_declared_only_by_layers_that_allow_it() {
    let mut violations = Vec::new();
    for member in members() {
        if member.layer().gpui_allowed {
            continue;
        }
        for dependency in &member.dependencies {
            if is_gpui(&dependency.package) || is_gpui(&dependency.key) {
                violations.push(format!(
                    "`{}` must not depend on `{}`",
                    member.name, dependency.key
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn every_member_inherits_the_workspace_lints() {
    let missing: Vec<&str> = members()
        .iter()
        .filter(|member| {
            let inherits = member
                .manifest
                .get("lints")
                .and_then(|lints| lints.get("workspace"))
                .and_then(toml::Value::as_bool);
            inherits != Some(true)
        })
        .map(|member| member.name.as_str())
        .collect();
    assert!(
        missing.is_empty(),
        "add `[lints] workspace = true` to: {missing:?}"
    );
}

#[test]
fn external_dependencies_come_from_the_workspace_table() {
    let registered = workspace_dependency_names();
    let mut violations = Vec::new();
    for member in members() {
        for dependency in external_dependencies(&member.dependencies) {
            if !dependency.inherits_workspace || !registered.contains(&dependency.key) {
                violations.push(format!(
                    "`{}` declares `{}` itself; use `workspace = true` and register it under \
                     [workspace.dependencies]",
                    member.name, dependency.key
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
