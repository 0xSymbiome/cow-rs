use policy_maintainer::{
    check_panic_allowlist::{
        PanicAllowlist, PanicAllowlistEntry, PanicGateError, check_entry_artifacts,
        validate_allowlist,
    },
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
        documented: None,
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

#[test]
fn rejects_allowlisted_item_missing_panics_rustdoc() {
    let fixture = r#"
        pub fn explode(x: u32) -> u32 {
            // SAFETY: checked arithmetic documents the local invariant.
            x.checked_add(1).expect("overflow")
        }
    "#;
    let entry = PanicAllowlistEntry {
        file: "test.rs".to_owned(),
        item: "explode".to_owned(),
        reason: "u32 overflow at boundary".to_owned(),
        documented: None,
    };

    let result = check_entry_artifacts(&entry, &syn::parse_file(fixture).unwrap(), fixture);

    assert!(matches!(
        result,
        Err(errs) if errs
            .iter()
            .any(|error| matches!(error, PanicGateError::MissingPanicsRustdoc { .. }))
    ));
}

#[test]
fn rejects_allowlisted_item_missing_safety_comment() {
    let fixture = r#"
        /// Computes the next index.
        ///
        /// # Panics
        ///
        /// Panics if `x == u32::MAX`.
        pub fn explode(x: u32) -> u32 {
            x.checked_add(1).expect("overflow")
        }
    "#;
    let entry = PanicAllowlistEntry {
        file: "test.rs".to_owned(),
        item: "explode".to_owned(),
        reason: "u32 overflow at boundary".to_owned(),
        documented: None,
    };

    let result = check_entry_artifacts(&entry, &syn::parse_file(fixture).unwrap(), fixture);

    assert!(matches!(
        result,
        Err(errs) if errs
            .iter()
            .any(|error| matches!(error, PanicGateError::MissingSafetyComment { .. }))
    ));
}

#[test]
fn accepts_item_with_both_artifacts() {
    let fixture = r#"
        /// Computes the next index.
        ///
        /// # Panics
        ///
        /// Panics if `x == u32::MAX`.
        pub fn explode(x: u32) -> u32 {
            // SAFETY: caller documented `x < u32::MAX` precondition.
            x.checked_add(1).expect("overflow")
        }
    "#;
    let entry = PanicAllowlistEntry {
        file: "test.rs".to_owned(),
        item: "explode".to_owned(),
        reason: "u32 overflow at boundary".to_owned(),
        documented: None,
    };

    let result = check_entry_artifacts(&entry, &syn::parse_file(fixture).unwrap(), fixture);

    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn accepts_impl_item_with_both_artifacts() {
    let fixture = r#"
        pub struct Address(String);

        impl Address {
            pub fn new(value: String) -> Self {
                Self(value)
            }

            /// Creates an address from bytes.
            ///
            /// # Panics
            ///
            /// Panics if the internal ASCII encoder stops emitting UTF-8.
            pub fn from_bytes(bytes: [u8; 20]) -> Self {
                let value = bytes.to_vec();
                // SAFETY: the encoder emits only ASCII bytes.
                String::from_utf8(value).expect("bytes are UTF-8");
                Self(String::new())
            }
        }

        pub struct Hash32(String);

        impl Hash32 {
            pub fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(format!("{bytes:?}"))
            }
        }
    "#;
    let entry = PanicAllowlistEntry {
        file: "test.rs".to_owned(),
        item: "Address::from_bytes".to_owned(),
        reason: "ASCII encoder invariant".to_owned(),
        documented: None,
    };

    let result = check_entry_artifacts(&entry, &syn::parse_file(fixture).unwrap(), fixture);

    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn accepts_item_with_documented_false_opt_out() {
    let fixture = r#"
        pub const fn const_helper(x: u32) -> u32 {
            assert!(x > 0);
            x - 1
        }
    "#;
    let entry = PanicAllowlistEntry {
        file: "test.rs".to_owned(),
        item: "const_helper".to_owned(),
        reason: "compile-time-checked precondition".to_owned(),
        documented: Some(false),
    };

    let result = check_entry_artifacts(&entry, &syn::parse_file(fixture).unwrap(), fixture);

    assert!(result.is_ok(), "{result:?}");
}
