//! Registry environment-scope contract test: assert the build-time
//! invariants reject capability rows under Prod or Staging and reject
//! `GPv2` rows under `EnvironmentAgnostic`, by inspecting the working
//! tree `registry.toml` directly.

use std::path::PathBuf;

const CAPABILITY_CONTRACTS: &[&str] = &[
    "ComposableCow",
    "ExtensibleFallbackHandler",
    "CurrentBlockTimestampFactory",
    "TwapHandler",
    "GoodAfterTimeHandler",
    "StopLossHandler",
    "TradeAboveThresholdHandler",
    "PerpetualStableSwapHandler",
    "CowShedImplementation",
    "CowShedFactory",
    "CowShedForComposableCow",
];

const GPV2_CONTRACTS: &[&str] = &["Settlement", "VaultRelayer", "EthFlow"];

fn registry_toml() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("registry.toml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn every_capability_row_is_environment_agnostic() {
    let toml = registry_toml();
    walk_entries(&toml, |contract, env| {
        if CAPABILITY_CONTRACTS.contains(&contract) {
            assert_eq!(
                env, "environment_agnostic",
                "capability row {contract} must be environment_agnostic; got {env}"
            );
        }
    });
}

#[test]
fn no_gpv2_row_is_environment_agnostic() {
    let toml = registry_toml();
    walk_entries(&toml, |contract, env| {
        if GPV2_CONTRACTS.contains(&contract) {
            assert!(
                env == "prod" || env == "staging",
                "GPv2 row {contract} must be prod or staging; got {env}"
            );
        }
    });
}

fn walk_entries(toml: &str, mut visit: impl FnMut(&str, &str)) {
    let mut current_contract = String::new();
    let mut current_env = String::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[entries]]" {
            if !current_contract.is_empty() && !current_env.is_empty() {
                visit(&current_contract, &current_env);
            }
            current_contract.clear();
            current_env.clear();
            continue;
        }
        if let Some(value) = trimmed
            .strip_prefix("contract_id = ")
            .map(|s| s.trim_matches('"'))
        {
            current_contract = value.to_string();
        }
        if let Some(value) = trimmed.strip_prefix("env = ").map(|s| s.trim_matches('"')) {
            current_env = value.to_string();
        }
    }
    if !current_contract.is_empty() && !current_env.is_empty() {
        visit(&current_contract, &current_env);
    }
}
