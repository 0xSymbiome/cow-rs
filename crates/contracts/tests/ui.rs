#[test]
fn non_exhaustive_signing_enums_reject_external_exhaustive_matches() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/non_exhaustive_external_match.rs");
}

#[test]
fn typestate_marker_types_reject_external_construction() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/typestate_marker_sealing.rs");
}
