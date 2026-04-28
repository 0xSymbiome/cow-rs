use policy_maintainer::{
    check_enum_policy::{EnumPolicy, EnumPolicyEntry, validate_policy},
    workspace::PublicEnum,
};

fn entry(marker: &str) -> EnumPolicyEntry {
    EnumPolicyEntry {
        name: "Mode".to_owned(),
        file: "crates/demo/src/lib.rs".to_owned(),
        line: Some(1),
        category: "sdk-local-state".to_owned(),
        expected_marker: marker.to_owned(),
        reason: "fixture enum classification".to_owned(),
    }
}

fn discovered(non_exhaustive: bool) -> PublicEnum {
    PublicEnum {
        file: "crates/demo/src/lib.rs".to_owned(),
        item: "Mode".to_owned(),
        name: "Mode".to_owned(),
        is_non_exhaustive: non_exhaustive,
    }
}

#[test]
fn enum_policy_accepts_matching_manifest_entry() {
    let policy = EnumPolicy {
        version: 1,
        enums: vec![entry("non_exhaustive")],
    };

    assert!(validate_policy(&policy, &[discovered(true)]).is_empty());
}

#[test]
fn enum_policy_reports_missing_and_marker_mismatch() {
    let missing = EnumPolicy {
        version: 1,
        enums: Vec::new(),
    };
    assert!(
        validate_policy(&missing, &[discovered(true)])[0].contains("missing enum policy entry")
    );

    let mismatch = EnumPolicy {
        version: 1,
        enums: vec![entry("exhaustive")],
    };
    assert!(validate_policy(&mismatch, &[discovered(true)])[0].contains("must remain exhaustive"));
}
