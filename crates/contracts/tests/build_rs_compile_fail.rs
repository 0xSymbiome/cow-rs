//! Compile-fail harness for the `registry.toml` validator.
//!
//! The `build.rs` gate and the runtime [`Registry::from_toml_str`] loader
//! share a single validation path, so exercising the loader against
//! deliberately-malformed manifest fixtures covers every diagnostic arm
//! `build.rs` rejects at compile time. Every fixture in the adjacent
//! `build_rs_compile_fail/` directory carries one shape a well-formed
//! manifest must avoid; the test walks the fixtures and asserts each
//! produces a typed [`RegistryError`] with the expected diagnostic text.
//!
//! Adding a new negative fixture alongside an entry in
//! [`EXPECTED_FAILURES`] keeps the compile-time gate and the runtime
//! loader in lockstep without requiring a custom CI workflow step.

use cow_sdk_contracts::{Registry, RegistryError};

struct Expected {
    fixture: &'static str,
    matcher: fn(&RegistryError) -> bool,
    description: &'static str,
}

const EXPECTED_FAILURES: &[Expected] = &[
    Expected {
        fixture: "bad_schema_version.toml",
        matcher: |error| matches!(error, RegistryError::UnsupportedSchemaVersion { .. }),
        description: "schema_version drift must reject the manifest",
    },
    Expected {
        fixture: "unknown_contract_id.toml",
        matcher: |error| matches!(error, RegistryError::Parse { .. }),
        description: "unknown contract_id variants must be rejected at parse time",
    },
    Expected {
        fixture: "unsupported_chain.toml",
        matcher: |error| matches!(error, RegistryError::UnsupportedChainId { .. }),
        description: "chain ids outside the supported set must be rejected",
    },
    Expected {
        fixture: "invalid_address.toml",
        matcher: |error| matches!(error, RegistryError::InvalidAddress { .. }),
        description: "malformed deployment addresses must be rejected",
    },
    Expected {
        fixture: "duplicate_entry.toml",
        matcher: |error| matches!(error, RegistryError::DuplicateEntry { .. }),
        description: "duplicate (ContractId, SupportedChainId, CowEnv) keys must be rejected",
    },
    Expected {
        fixture: "malformed_syntax.toml",
        matcher: |error| matches!(error, RegistryError::Parse { .. }),
        description: "TOML syntax errors must surface through the typed parser",
    },
];

fn fixture_source(name: &str) -> &'static str {
    // Each fixture is inlined via `include_str!` so the compile-fail harness
    // remains self-contained and does not depend on a filesystem walk at
    // test-run time.
    match name {
        "bad_schema_version.toml" => {
            include_str!("build_rs_compile_fail/bad_schema_version.toml")
        }
        "unknown_contract_id.toml" => {
            include_str!("build_rs_compile_fail/unknown_contract_id.toml")
        }
        "unsupported_chain.toml" => {
            include_str!("build_rs_compile_fail/unsupported_chain.toml")
        }
        "invalid_address.toml" => include_str!("build_rs_compile_fail/invalid_address.toml"),
        "duplicate_entry.toml" => include_str!("build_rs_compile_fail/duplicate_entry.toml"),
        "malformed_syntax.toml" => include_str!("build_rs_compile_fail/malformed_syntax.toml"),
        other => panic!("no inlined source for fixture `{other}`"),
    }
}

#[test]
fn malformed_registry_fixtures_produce_typed_errors() {
    for expected in EXPECTED_FAILURES {
        let source = fixture_source(expected.fixture);
        let Err(error) = Registry::from_toml_str(source) else {
            panic!(
                "fixture `{}` unexpectedly parsed cleanly; expected {}",
                expected.fixture, expected.description,
            );
        };
        assert!(
            (expected.matcher)(&error),
            "fixture `{}` did not match the expected diagnostic arm: {} (got {error:?})",
            expected.fixture,
            expected.description,
        );
    }
}
