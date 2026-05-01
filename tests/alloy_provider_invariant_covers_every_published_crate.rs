use std::{
    collections::BTreeSet,
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

fn string_literals_in_shipped_crates_const(source: &str) -> BTreeSet<String> {
    let start = source
        .find("const SHIPPED_CRATES")
        .expect("policy maintainer must declare SHIPPED_CRATES");
    let body = &source[start..];
    let end = body
        .find("];")
        .expect("SHIPPED_CRATES must be a string slice constant");
    body[..end]
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let value = trimmed.strip_prefix('"')?.split('"').next()?;
            Some(value.to_owned())
        })
        .collect()
}

#[test]
fn alloy_provider_invariant_covers_every_published_crate() {
    let root = repo_root();
    let root_manifest = read_toml(&root.join("Cargo.toml"));
    let members = root_manifest["workspace"]["members"]
        .as_array()
        .expect("workspace members must be declared");
    let mut published_crates = BTreeSet::new();

    for member in members.iter().filter_map(Value::as_str) {
        if !member.starts_with("crates/") {
            continue;
        }
        let manifest = read_toml(&root.join(member).join("Cargo.toml"));
        let package = manifest["package"]
            .as_table()
            .expect("crate member must declare [package]");
        if package.get("publish").and_then(Value::as_bool) == Some(false) {
            continue;
        }
        let name = package["name"]
            .as_str()
            .expect("crate member must declare package.name");
        published_crates.insert(name.to_owned());
    }

    let policy_source = fs::read_to_string(
        root.join("scripts/policy-maintainer/src/check_alloy_provider_invariant.rs"),
    )
    .expect("policy maintainer alloy invariant source must be readable");
    let enumerated = string_literals_in_shipped_crates_const(&policy_source);

    assert_eq!(
        enumerated, published_crates,
        "cargo check-alloy-provider-invariant must enumerate every published workspace crate under crates/",
    );
}
