//! Which crates may depend on which, and which external crates each layer may use.

use architecture::{Member, members, path_targets, workspace_dependency_names};

/// Workspace crates a member may depend on.
fn allowed_internal(name: &str) -> &'static [&'static str] {
    match name {
        "myapp-domain" | "myapp-ui" | "architecture" => &[],
        "myapp-core" => &["myapp-domain"],
        "myapp-storage" => &["myapp-core"],
        "myapp" => &["myapp-core", "myapp-storage", "myapp-ui"],
        other => panic!("unknown workspace member `{other}`; register it in tests/architecture"),
    }
}

/// External crates a layer may use. The composition root has no list.
fn allowed_external(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "myapp-domain" => Some(&["thiserror"]),
        "myapp-core" => Some(&["thiserror", "tracing"]),
        "myapp-storage" => Some(&["serde", "serde_json", "tempfile"]),
        "myapp-ui" => Some(&["gpui"]),
        "architecture" => Some(&["toml"]),
        "myapp" => None,
        other => panic!("unknown workspace member `{other}`; register it in tests/architecture"),
    }
}

fn external_dependencies(member: &Member) -> impl Iterator<Item = &architecture::Dependency> {
    member
        .dependencies
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
    for member in &all {
        let allowed = allowed_internal(&member.name);
        let (targets, unresolved) = path_targets(member, &all);
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
fn inner_layers_use_only_allow_listed_external_crates() {
    let mut violations = Vec::new();
    for member in members() {
        let Some(allowed) = allowed_external(&member.name) else {
            continue;
        };
        for dependency in external_dependencies(&member) {
            if !allowed.contains(&dependency.package.as_str()) {
                violations.push(format!(
                    "`{}` may not depend on `{}`; allowed: {allowed:?}",
                    member.name, dependency.package
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn gpui_is_confined_to_the_app_and_the_ui_kit() {
    let mut violations = Vec::new();
    for member in members() {
        if matches!(member.name.as_str(), "myapp" | "myapp-ui") {
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
    let missing: Vec<String> = members()
        .into_iter()
        .filter(|member| {
            let inherits = member
                .manifest
                .get("lints")
                .and_then(|lints| lints.get("workspace"))
                .and_then(toml::Value::as_bool);
            inherits != Some(true)
        })
        .map(|member| member.name)
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
        for dependency in external_dependencies(&member) {
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
