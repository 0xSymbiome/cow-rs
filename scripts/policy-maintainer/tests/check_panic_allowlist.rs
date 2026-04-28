use policy_maintainer::{
    check_panic_allowlist::{PanicAllowlist, PanicAllowlistEntry, validate_allowlist},
    workspace::PanicCall,
};

fn call() -> PanicCall {
    PanicCall {
        file: "crates/demo/src/lib.rs".to_owned(),
        item: "build".to_owned(),
        kind: "expect".to_owned(),
    }
}

fn entry(item: &str) -> PanicAllowlistEntry {
    PanicAllowlistEntry {
        file: "crates/demo/src/lib.rs".to_owned(),
        item: item.to_owned(),
        reason: "fixture invariant".to_owned(),
    }
}

#[test]
fn panic_allowlist_accepts_item_path_match() {
    let allowlist = PanicAllowlist {
        version: 1,
        allowed: vec![entry("build")],
    };

    assert!(validate_allowlist(&allowlist, &[call()]).is_empty());
}

#[test]
fn panic_allowlist_reports_missing_and_stale_entries() {
    let missing = PanicAllowlist {
        version: 1,
        allowed: Vec::new(),
    };
    assert!(validate_allowlist(&missing, &[call()])[0].contains("not allowlisted"));

    let stale = PanicAllowlist {
        version: 1,
        allowed: vec![entry("stale")],
    };
    let errors = validate_allowlist(&stale, &[]);
    assert!(errors[0].contains("no matching panic-bearing call"));
}
