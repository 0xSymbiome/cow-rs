use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use toml::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live under the repository root")
        .to_path_buf()
}

fn read_toml(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    toml::from_str(&content).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

fn dependency_version(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.as_table()?.get("version")?.as_str())
}

fn uses_workspace_dependency(value: &Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        == Some(true)
}

fn major_minor(version: &str) -> String {
    let normalized = version.trim_start_matches('=');
    let mut parts = normalized.split('.');
    let major = parts.next().expect("dependency version must carry a major");
    let minor = parts.next().expect("dependency version must carry a minor");
    format!("{major}.{minor}")
}

fn cargo_manifests_under(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut manifests = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", dir.display());
        }) {
            let entry = entry.expect("directory entry must be readable");
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| {
                    matches!(
                        name.to_str(),
                        Some(".git" | ".local" | ".agents" | "target" | "node_modules" | "tmp")
                    )
                }) {
                    continue;
                }
                pending.push(path);
            } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
                manifests.push(path);
            }
        }
    }
    manifests
}

#[test]
fn workspace_alloy_pin_lockstep() {
    let root = repo_root();
    let root_manifest = read_toml(&root.join("Cargo.toml"));
    let root_dependencies = root_manifest["workspace"]["dependencies"]
        .as_table()
        .expect("root workspace dependencies must be a table");
    let root_alloy: BTreeMap<_, _> = root_dependencies
        .iter()
        .filter(|(name, _)| name.starts_with("alloy-"))
        .map(|(name, value)| {
            let version = dependency_version(value)
                .unwrap_or_else(|| panic!("root {name} dependency must declare a version"));
            (name.as_str(), major_minor(version))
        })
        .collect();

    assert!(
        !root_alloy.is_empty(),
        "root workspace must declare reviewed alloy-* pins",
    );

    for manifest_path in cargo_manifests_under(&root) {
        if manifest_path == root.join("Cargo.toml") {
            continue;
        }
        let manifest = read_toml(&manifest_path);
        let Some(dependencies) = manifest.get("dependencies").and_then(Value::as_table) else {
            continue;
        };

        for (name, value) in dependencies
            .iter()
            .filter(|(name, _)| name.starts_with("alloy-"))
        {
            if uses_workspace_dependency(value) {
                continue;
            }
            let expected = root_alloy
                .get(name.as_str())
                .unwrap_or_else(|| panic!("{name} must be pinned at the root workspace"));
            let version = dependency_version(value).unwrap_or_else(|| {
                panic!("{} must declare an alloy version", manifest_path.display());
            });
            assert_eq!(
                &major_minor(version),
                expected,
                "{} must keep {name} on the root alloy major.minor line",
                manifest_path.display(),
            );
        }
    }
}
