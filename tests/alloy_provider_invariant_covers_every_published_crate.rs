use std::collections::BTreeSet;

#[path = "support/published_crates.rs"]
mod support;

use support::{dependency_names, published_workspace_crates, read_toml, repo_root};

#[test]
fn alloy_provider_invariant_covers_every_published_crate() {
    let root = repo_root();
    let published_crates = published_workspace_crates(&root);
    let allowlist = BTreeSet::from([
        "cow-sdk-alloy-provider".to_owned(),
        "cow-sdk-alloy".to_owned(),
    ]);

    let alloy_provider_users: BTreeSet<_> = published_crates
        .iter()
        .filter_map(|(name, manifest_path)| {
            let manifest = read_toml(manifest_path);
            dependency_names(&manifest)
                .contains("alloy-provider")
                .then_some(name.clone())
        })
        .collect();

    assert_eq!(
        alloy_provider_users, allowlist,
        "only the native alloy provider leaf and umbrella may depend on alloy-provider",
    );
}
