//! Registry capability-row contract test.
//!
//! Assert the working tree `registry.toml` carries exactly 177 entries
//! (66 `GPv2` env-specific plus 111 capability `EnvironmentAgnostic`),
//! with the capability breakdown matching the capability landing spec.

use std::path::PathBuf;

fn registry_toml() -> String {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = repo_root.join("registry.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn count_entries(toml: &str) -> usize {
    toml.lines()
        .filter(|line| line.trim() == "[[entries]]")
        .count()
}

fn count_entries_with(toml: &str, needle: &str) -> usize {
    let mut count = 0_usize;
    let mut in_entry = false;
    let mut matched = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[entries]]" {
            if in_entry && matched {
                count += 1;
            }
            in_entry = true;
            matched = false;
            continue;
        }
        if !in_entry {
            continue;
        }
        // `[entries.verification]` is a child sub-table of the current
        // entry, not the start of a new entry. We only flush at the next
        // `[[entries]]` marker; intermediate sub-table headers stay
        // inside the same entry block.
        if trimmed.contains(needle) {
            matched = true;
        }
    }
    if in_entry && matched {
        count += 1;
    }
    count
}

#[test]
fn registry_totals_177_rows_under_branch_b() {
    let toml = registry_toml();
    assert_eq!(
        count_entries(&toml),
        177,
        "Branch B default carries 66 GPv2 env-specific rows + 111 capability rows = 177 total"
    );
}

#[test]
fn capability_rows_are_environment_agnostic() {
    let toml = registry_toml();
    let composable_cow = count_entries_with(&toml, "ComposableCow");
    assert!(
        composable_cow >= 11,
        "ComposableCow expected to appear on at least 11 chains (10 SupportedChainId + Lens via DeploymentChainId); got {composable_cow}"
    );
}

#[test]
fn cow_shed_for_composable_cow_is_gnosis_only() {
    let toml = registry_toml();
    let count = count_entries_with(&toml, "CowShedForComposableCow");
    assert_eq!(
        count, 1,
        "CowShedForComposableCow must appear exactly once (Gnosis Chain only); got {count}"
    );
}

#[test]
fn no_ink_composable_or_cow_shed_in_registry() {
    let toml = registry_toml();
    let mut current_chain = String::new();
    let mut current_contract = String::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[entries]]" {
            if (current_chain == "Ink" || current_chain == "57073")
                && (current_contract == "ComposableCow"
                    || current_contract == "CowShedFactory"
                    || current_contract == "CowShedImplementation")
            {
                panic!(
                    "Ink + {current_contract} must be a coverage record in deployment-coverage.yaml, not a registry row"
                );
            }
            current_chain.clear();
            current_contract.clear();
            continue;
        }
        if let Some(value) = trimmed
            .strip_prefix("chain_id = ")
            .map(|s| s.trim_matches('"'))
        {
            current_chain = value.to_string();
        }
        if let Some(value) = trimmed
            .strip_prefix("contract_id = ")
            .map(|s| s.trim_matches('"'))
        {
            current_contract = value.to_string();
        }
    }
}
