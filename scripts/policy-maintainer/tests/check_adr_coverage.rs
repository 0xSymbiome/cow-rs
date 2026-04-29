use policy_maintainer::check_adr_coverage::{
    AdrStatus, PrincipleAdrEntry, PrincipleAdrMap, validate_coverage,
};

fn map() -> PrincipleAdrMap {
    PrincipleAdrMap {
        version: 1,
        principles: vec![PrincipleAdrEntry {
            id: 1,
            name: "Deterministic Protocol Transforms".to_owned(),
            primary_adr: "0001".to_owned(),
            supporting_adrs: vec!["0002".to_owned()],
        }],
        out_of_scope_adrs: Vec::new(),
    }
}

#[test]
fn adr_coverage_accepts_mapped_accepted_adrs() {
    let statuses = vec![
        AdrStatus {
            id: "0001".to_owned(),
            path: "docs/adr/0001.md".to_owned(),
            status: "Accepted (amended)".to_owned(),
        },
        AdrStatus {
            id: "0002".to_owned(),
            path: "docs/adr/0002.md".to_owned(),
            status: "Accepted".to_owned(),
        },
    ];

    assert!(validate_coverage(&map(), &statuses).is_empty());
}

#[test]
fn adr_coverage_reports_missing_primary_and_unmapped_accepted_adr() {
    let statuses = vec![AdrStatus {
        id: "0003".to_owned(),
        path: "docs/adr/0003.md".to_owned(),
        status: "Accepted".to_owned(),
    }];

    let errors = validate_coverage(&map(), &statuses);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("missing ADR 0001"))
    );
    assert!(errors.iter().any(|error| error.contains("not mapped")));
}
