use clap::{Parser, Subcommand};
use policy_maintainer::{
    check_adr_coverage, check_alloy_provider_invariant, check_chain_patch_eligibility,
    check_deny_unknown_fields, check_enum_policy, check_msrv_notice, check_panic_allowlist,
    check_property_citations, check_source_lock_roots, check_stub, check_wasm_runner_freshness,
    check_workspace_versions, classify_release, diagnostics::OutputMode,
    generate_validation_evidence, run_deterministic_examples,
};

#[derive(Debug, Parser)]
#[command(
    name = "policy-maintainer",
    version,
    about = "Run cow-rs policy maintenance checks."
)]
struct Cli {
    /// Emit diagnostics as newline-delimited JSON.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Classify a release transition and emit the semver-checks dispatch contract.
    #[command(name = "classify-release")]
    ClassifyRelease(classify_release::Args),
    /// Verify every public enum is classified in the enum policy manifest.
    #[command(name = "check-enum-policy")]
    CheckEnumPolicy(check_enum_policy::Args),
    /// Verify panic-bearing production source calls are explicitly allowlisted.
    #[command(name = "check-panic-allowlist")]
    CheckPanicAllowlist(check_panic_allowlist::Args),
    /// Verify serde(deny_unknown_fields) only appears on allowlisted SDK-owned DTOs.
    #[command(name = "check-deny-unknown-fields")]
    CheckDenyUnknownFields(check_deny_unknown_fields::Args),
    /// Verify workspace crate versions follow the active release alignment policy.
    #[command(name = "check-workspace-versions")]
    CheckWorkspaceVersions(check_workspace_versions::Args),
    /// Verify the workspace MSRV bump notice window.
    #[command(name = "check-msrv-notice")]
    CheckMsrvNotice(check_msrv_notice::Args),
    /// Verify shipped crates do not depend on alloy-provider.
    #[command(name = "check-alloy-provider-invariant")]
    CheckAlloyProviderInvariant(check_alloy_provider_invariant::Args),
    /// Verify principles and accepted ADRs are mutually covered.
    #[command(name = "check-adr-coverage")]
    CheckAdrCoverage(check_adr_coverage::Args),
    /// Verify patch-lane chain additions do not require source-lock refresh.
    #[command(name = "check-chain-patch-eligibility")]
    CheckChainPatchEligibility(check_chain_patch_eligibility::Args),
    /// Verify pinned WASM browser runner versions are fresh enough.
    #[command(name = "check-wasm-runner-freshness")]
    CheckWasmRunnerFreshness(check_wasm_runner_freshness::Args),
    /// Verify property registry citations resolve to real test functions.
    #[command(name = "check-property-citations")]
    CheckPropertyCitations(check_property_citations::Args),
    /// Verify source-lock local roots match their pinned upstream repositories.
    #[command(name = "check-source-lock-roots")]
    CheckSourceLockRoots(check_source_lock_roots::Args),
    /// Generate or check the release validation evidence artefact.
    #[command(name = "generate-validation-evidence")]
    GenerateValidationEvidence(generate_validation_evidence::Args),
    /// Execute every deterministic non-live example binary.
    #[command(name = "run-deterministic-examples")]
    RunDeterministicExamples(run_deterministic_examples::Args),
    /// Run the policy-maintainer skeleton smoke check.
    #[command(name = "check-stub")]
    CheckStub(check_stub::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output_mode = OutputMode::from_json(cli.json);

    match cli.command {
        Command::ClassifyRelease(args) => classify_release::run(args, output_mode),
        Command::CheckEnumPolicy(args) => check_enum_policy::run(args, output_mode),
        Command::CheckPanicAllowlist(args) => check_panic_allowlist::run(args, output_mode),
        Command::CheckDenyUnknownFields(args) => check_deny_unknown_fields::run(args, output_mode),
        Command::CheckWorkspaceVersions(args) => check_workspace_versions::run(args, output_mode),
        Command::CheckMsrvNotice(args) => check_msrv_notice::run(args, output_mode),
        Command::CheckAlloyProviderInvariant(args) => {
            check_alloy_provider_invariant::run(args, output_mode)
        }
        Command::CheckAdrCoverage(args) => check_adr_coverage::run(args, output_mode),
        Command::CheckChainPatchEligibility(args) => {
            check_chain_patch_eligibility::run(args, output_mode)
        }
        Command::CheckWasmRunnerFreshness(args) => {
            check_wasm_runner_freshness::run(args, output_mode)
        }
        Command::CheckPropertyCitations(args) => check_property_citations::run(args, output_mode),
        Command::CheckSourceLockRoots(args) => check_source_lock_roots::run(args, output_mode),
        Command::GenerateValidationEvidence(args) => {
            generate_validation_evidence::run(args, output_mode)
        }
        Command::RunDeterministicExamples(args) => {
            run_deterministic_examples::run(args, output_mode)
        }
        Command::CheckStub(args) => check_stub::run(args, output_mode),
    }
}
