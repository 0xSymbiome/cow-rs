use xtask::policy::{
    check_deny_unknown_fields::{
        DenyUnknownFieldsAllowlist, DenyUnknownFieldsEntry, validate_allowlist,
    },
    workspace::DenyUnknownFields,
};

fn occurrence() -> DenyUnknownFields {
    DenyUnknownFields {
        file: "crates/demo/src/lib.rs".to_owned(),
        item: "Request".to_owned(),
    }
}

fn entry(item: &str) -> DenyUnknownFieldsEntry {
    DenyUnknownFieldsEntry {
        file: "crates/demo/src/lib.rs".to_owned(),
        item: item.to_owned(),
        reason: "fixture schema is SDK-owned".to_owned(),
    }
}

#[test]
fn deny_unknown_fields_allowlist_accepts_matching_item() {
    let allowlist = DenyUnknownFieldsAllowlist {
        version: 1,
        allowed: vec![entry("Request")],
    };

    assert!(validate_allowlist(&allowlist, &[occurrence()]).is_empty());
}

#[test]
fn deny_unknown_fields_allowlist_reports_unmatched_occurrence_and_stale_entry() {
    let missing = DenyUnknownFieldsAllowlist {
        version: 1,
        allowed: Vec::new(),
    };
    assert!(validate_allowlist(&missing, &[occurrence()])[0].contains("not allowlisted"));

    let stale = DenyUnknownFieldsAllowlist {
        version: 1,
        allowed: vec![entry("Stale")],
    };
    assert!(validate_allowlist(&stale, &[])[0].contains("no matching item"));
}
