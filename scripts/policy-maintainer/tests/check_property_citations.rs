mod common;

use common::TempDir;
use policy_maintainer::check_property_citations::{
    PropertyRow, evidence_refs, parse_property_rows, validate_rows,
};

#[test]
fn property_citations_accept_real_test_function_symbols() {
    let temp = TempDir::new("property-citations-pass");
    temp.write(
        "crates/demo/tests/contract.rs",
        r#"
#[test]
fn cited_contract_holds() {}
"#,
    );
    let rows = vec![PropertyRow {
        id: "PROP-DEMO-001".to_owned(),
        covered: "Yes".to_owned(),
        evidence: "`crates/demo/tests/contract.rs::cited_contract_holds`".to_owned(),
    }];

    assert!(validate_rows(temp.path(), &rows).unwrap().is_empty());
}

#[test]
fn property_citations_reject_missing_or_non_test_symbols() {
    let temp = TempDir::new("property-citations-fail");
    temp.write(
        "crates/demo/tests/contract.rs",
        r#"
fn helper_only() {}
"#,
    );
    let rows = vec![PropertyRow {
        id: "PROP-DEMO-001".to_owned(),
        covered: "Yes".to_owned(),
        evidence: "`crates/demo/tests/contract.rs::helper_only`".to_owned(),
    }];

    let errors = validate_rows(temp.path(), &rows).unwrap();
    assert!(errors[0].contains("missing or is not a test function"));
}

#[test]
fn property_table_parser_extracts_prop_rows_and_rust_refs() {
    let rows = parse_property_rows(
        "| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |\n\
         | --- | --- | --- | --- | --- | --- | --- |\n\
         | `PROP-DEMO-001` | `demo` | Holds | Contract | Yes | `crates/demo/tests/contract.rs::cited_contract_holds` | 2026-04-28 |\n",
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(
        evidence_refs(&rows[0].evidence)[0].symbol.as_deref(),
        Some("cited_contract_holds")
    );
}
