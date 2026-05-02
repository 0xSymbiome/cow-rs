use std::{collections::BTreeMap, fs, path::PathBuf};

use cow_sdk_core::SupportedChainId;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live under the repository root")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn supported_networks_doc_table_matches_enum() {
    let doc = read_repo_file("docs/parity-matrix.md");
    let documented = parse_supported_networks_table(&doc);
    let actual = SupportedChainId::ALL
        .iter()
        .map(|chain| (format!("{chain:?}"), *chain as u64))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        documented, actual,
        "docs/parity-matrix.md Supported Networks table drifted from \
         SupportedChainId; update either the docs or the enum."
    );
}

fn parse_supported_networks_table(doc: &str) -> BTreeMap<String, u64> {
    let supported_heading = doc
        .lines()
        .position(|line| line.trim() == "## Supported Networks")
        .expect("docs/parity-matrix.md must contain a Supported Networks section");

    let mut rows = doc
        .lines()
        .skip(supported_heading + 1)
        .take_while(|line| !line.trim_start().starts_with("## "))
        .skip_while(|line| !line.trim_start().starts_with("| Chain |"));

    let header = rows
        .next()
        .expect("Supported Networks section must contain a markdown table header");
    assert!(
        header.contains("`SupportedChainId` variant")
            && header.contains("Numeric chain id")
            && header.contains("Environment"),
        "Supported Networks table header must document variant, numeric id, and environment"
    );

    let separator = rows
        .next()
        .expect("Supported Networks table must contain a separator row");
    assert!(
        separator
            .split('|')
            .all(|cell| cell.trim().is_empty() || cell.trim().chars().all(is_separator_char)),
        "Supported Networks table separator row is malformed"
    );

    let mut documented = BTreeMap::new();
    for row in rows.take_while(|line| line.trim_start().starts_with('|')) {
        let cells = row
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        assert!(
            cells.len() >= 4,
            "Supported Networks row must have at least four columns: {row}"
        );

        let variant = cells[1].trim_matches('`').to_owned();
        let chain_id = cells[2]
            .replace('_', "")
            .parse::<u64>()
            .unwrap_or_else(|error| panic!("invalid numeric chain id in row `{row}`: {error}"));

        assert!(
            documented.insert(variant.clone(), chain_id).is_none(),
            "duplicate SupportedChainId variant documented: {variant}"
        );
    }

    assert!(
        !documented.is_empty(),
        "Supported Networks table must contain at least one documented chain"
    );
    documented
}

const fn is_separator_char(candidate: char) -> bool {
    matches!(candidate, '-' | ':' | ' ')
}
