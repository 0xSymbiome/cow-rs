use xtask::policy::classify_release::{
    ReleaseKind, SemverChecksMode, classify_versions, workspace_version_from_toml,
};

#[test]
fn classification_table_matches_versioning_policy() {
    let cases = [
        (
            Some("0.0.1-reserved.0"),
            "0.1.0",
            ReleaseKind::FirstFunctional,
            SemverChecksMode::Skip,
            None,
        ),
        (
            None,
            "0.1.0",
            ReleaseKind::FirstFunctional,
            SemverChecksMode::Skip,
            None,
        ),
        (
            Some("0.1.0"),
            "0.1.1",
            ReleaseKind::Patch,
            SemverChecksMode::Blocking,
            Some("v0.1.0"),
        ),
        (
            Some("0.1.1"),
            "0.2.0",
            ReleaseKind::Pre1_0Minor,
            SemverChecksMode::Advisory,
            Some("v0.1.1"),
        ),
        (
            Some("0.2.3"),
            "0.2.4",
            ReleaseKind::Patch,
            SemverChecksMode::Blocking,
            Some("v0.2.3"),
        ),
        (
            Some("0.9.0"),
            "1.0.0",
            ReleaseKind::Major,
            SemverChecksMode::Skip,
            None,
        ),
        (
            Some("1.0.0"),
            "1.0.1",
            ReleaseKind::Patch,
            SemverChecksMode::Blocking,
            Some("v1.0.0"),
        ),
        (
            Some("1.0.1"),
            "1.1.0",
            ReleaseKind::Post1_0Minor,
            SemverChecksMode::Blocking,
            Some("v1.0.1"),
        ),
        (
            Some("1.9.9"),
            "2.0.0",
            ReleaseKind::Major,
            SemverChecksMode::Skip,
            None,
        ),
        (
            Some("0.1.0"),
            "0.3.0",
            ReleaseKind::Unsupported,
            SemverChecksMode::Skip,
            None,
        ),
    ];

    for (base, head, kind, mode, baseline) in cases {
        let classification = classify_versions(base, head).unwrap();
        assert_eq!(classification.release_kind, kind, "{base:?} -> {head}");
        assert_eq!(
            classification.semver_checks_mode, mode,
            "{base:?} -> {head}"
        );
        assert_eq!(
            classification.baseline_tag.as_deref(),
            baseline,
            "{base:?} -> {head}"
        );
    }
}

#[test]
fn workspace_version_parser_reads_workspace_package_version() {
    let version = workspace_version_from_toml(
        "[workspace]\r\nmembers = []\r\n[workspace.package]\r\nversion = \"0.2.3\"\r\n",
    )
    .unwrap();

    assert_eq!(version, "0.2.3");
}
