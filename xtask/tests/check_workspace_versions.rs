use xtask::policy::check_workspace_versions::{MemberVersion, validate_versions};

fn member(uses_workspace_version: bool, explicit_version: Option<&str>) -> MemberVersion {
    MemberVersion {
        manifest: "crates/demo/Cargo.toml".to_owned(),
        package_name: "cow-sdk-demo".to_owned(),
        uses_workspace_version,
        explicit_version: explicit_version.map(str::to_owned),
    }
}

#[test]
fn pre_1_0_workspace_requires_workspace_version_in_members() {
    assert!(validate_versions("0.1.0", &[member(true, None)]).is_empty());

    let errors = validate_versions("0.1.0", &[member(false, Some("0.1.0"))]);
    assert!(errors[0].contains("version.workspace = true"));
}

#[test]
fn post_1_0_workspace_allows_patch_divergence_only() {
    assert!(validate_versions("1.2.0", &[member(false, Some("1.2.3"))]).is_empty());

    let errors = validate_versions("1.2.0", &[member(false, Some("1.3.0"))]);
    assert!(errors[0].contains("not a patch divergence"));
}
