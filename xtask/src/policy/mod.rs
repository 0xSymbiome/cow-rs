//! Repository policy checks behind `cargo xtask policy`.
//!
//! Enum/panic/DTO allowlists, dependency and version invariants, ADR
//! coverage, property citations, and the deterministic-example runner — each
//! submodule is one subcommand, reachable through its stable `cargo check-*`
//! alias.

pub mod check_adr_coverage;
pub mod check_alloy_family_pins;
pub mod check_audit_freshness;
pub mod check_audit_lane;
pub mod check_chain_patch_eligibility;
pub mod check_deny_unknown_fields;
pub mod check_enum_policy;
pub mod check_msrv_notice;
pub mod check_panic_allowlist;
pub mod check_principles;
pub mod check_property_citations;
pub mod check_readme_include;
pub mod check_shell_wrappers;
pub mod check_wasm_invariant;
pub mod check_workflow_security;
pub mod check_workspace_versions;
pub mod dependency_invariant;
pub mod fences;
// Internal library dependency of `check_chain_patch_eligibility`; the
// standalone classify-release subcommand was retired upstream of this move.
pub mod classify_release;
pub mod fixtures;
pub mod run_deterministic_examples;
pub mod workspace;

use anyhow::bail;

/// A repository-state policy check reachable through `cargo check-policies`.
type Check = (&'static str, fn() -> anyhow::Result<()>);

/// Every repository-state policy check the `policy all` sweep runs, with its
/// CI-shaped default arguments.
///
/// Two policy subcommands stay outside this sweep by design:
/// `check-chain-patch-eligibility` needs a pull-request diff (base and head
/// refs), and `run-deterministic-examples` executes example binaries rather
/// than checking repository state.
const REPO_STATE_CHECKS: &[Check] = &[
    ("check-enum-policy", check_enum_policy::run_default),
    ("check-panic-allowlist", check_panic_allowlist::run_default),
    (
        "check-deny-unknown-fields",
        check_deny_unknown_fields::run_default,
    ),
    (
        "check-workspace-versions",
        check_workspace_versions::run_default,
    ),
    ("check-msrv-notice", check_msrv_notice::run_default),
    (
        "check-alloy-provider-invariant",
        dependency_invariant::run_alloy_provider_default,
    ),
    (
        "check-alloy-signer-invariant",
        dependency_invariant::run_alloy_signer_default,
    ),
    ("check-adr-coverage", check_adr_coverage::run_default),
    ("check-principles", check_principles::run_default),
    ("check-audit-lane", check_audit_lane::run_default),
    (
        "check-alloy-family-pins",
        check_alloy_family_pins::run_default,
    ),
    (
        "check-property-citations",
        check_property_citations::run_default,
    ),
    ("check-wasm-invariant", check_wasm_invariant::run_default),
    ("check-source-fences", fences::run_default),
    (
        "check-workflow-security",
        check_workflow_security::run_default,
    ),
    ("check-shell-wrappers", check_shell_wrappers::run_default),
    ("check-readme-include", check_readme_include::run_default),
];

/// Runs every repository-state policy check, summarizing failures at the end.
pub fn run_all() -> anyhow::Result<()> {
    let mut failures = Vec::new();
    for (name, check) in REPO_STATE_CHECKS {
        println!("==> {name}");
        if let Err(error) = check() {
            eprintln!("{name} failed: {error:#}");
            failures.push(*name);
        }
    }
    if failures.is_empty() {
        println!("all {} policy checks passed", REPO_STATE_CHECKS.len());
        Ok(())
    } else {
        bail!(
            "{} of {} policy checks failed: {}",
            failures.len(),
            REPO_STATE_CHECKS.len(),
            failures.join(", ")
        );
    }
}
