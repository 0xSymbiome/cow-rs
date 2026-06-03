//! Shared implementation for the `policy-maintainer` command-line tool.

pub mod check_adr_coverage;
pub mod check_alloy_provider_invariant;
pub mod check_alloy_signer_invariant;
pub mod check_chain_patch_eligibility;
pub mod check_deny_unknown_fields;
pub mod check_enum_policy;
pub mod check_msrv_notice;
pub mod check_panic_allowlist;
pub mod check_property_citations;
pub mod check_source_lock_roots;
pub mod check_wasm_invariant;
pub mod check_wasm_runner_freshness;
pub mod check_workspace_versions;
// Retained as an internal library dependency of `check_chain_patch_eligibility`;
// the standalone `classify-release` subcommand and its semver-checks gate were removed.
pub mod classify_release;
pub mod diagnostics;
pub mod fixtures;
pub mod run_deterministic_examples;
pub mod workspace;
