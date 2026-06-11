//! Dispatch-only entry point for the workspace maintenance tasks.
//!
//! Every capability lives in the [`xtask`] library ([`xtask::parity`],
//! [`xtask::policy`]) so the test suite exercises the same code CI runs; the
//! stable interface is the cargo alias set in `.cargo/config.toml`.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;
use xtask::docs::{agree, audit_index};
use xtask::parity::{self, openapi_coverage, registry_confirm, sync, vendor_openapi};
use xtask::policy::{
    check_adr_coverage, check_alloy_family_pins, check_chain_patch_eligibility,
    check_deny_unknown_fields, check_enum_policy, check_msrv_notice, check_panic_allowlist,
    check_property_citations, check_readme_include, check_shell_wrappers, check_wasm_invariant,
    check_workflow_security, check_workspace_versions, dependency_invariant, fences,
    run_deterministic_examples,
};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(
    about = "Workspace maintenance tasks: source-lock provenance, OpenAPI coverage, the deployment-registry probe, and repository policy checks"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Upstream-parity provenance tasks.
    #[command(subcommand)]
    Parity(ParityCommand),
    /// Repository policy checks.
    Policy(PolicyCli),
    /// Documentation-agreement gates.
    #[command(subcommand)]
    Docs(DocsCommand),
}

#[derive(Debug, Subcommand)]
enum DocsCommand {
    /// Verify the published release-gate commands agree across docs and CI.
    Agree(agree::Args),
    /// Verify the audit index review dates match the per-audit banners.
    AuditIndex(audit_index::Args),
}

#[derive(Debug, Subcommand)]
enum ParityCommand {
    /// Validate source-lock and committed parity fixture provenance.
    Validate(ValidateArgs),
    /// Vendor the source-lock-pinned services orderbook `OpenAPI` document.
    VendorOpenapi(vendor_openapi::VendorOpenApiArgs),
    /// Generate or validate `OpenAPI` DTO coverage inventories.
    OpenapiCoverage(openapi_coverage::OpenApiCoverageArgs),
    /// Confirm deployment provenance against live chain bytecode.
    ConfirmDeployments(ConfirmDeploymentsArgs),
    /// Materialize the pinned upstream checkouts (or advance the pins with --update).
    Sync(sync::SyncArgs),
    /// Report producer-path drift between the pins and an upstream ref.
    Drift(sync::DriftArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Args)]
struct ConfirmDeploymentsArgs {
    /// Output format for the confirmation report.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    #[command(flatten)]
    confirm: registry_confirm::RegistryConfirmArgs,
}

#[derive(Debug, Args)]
struct SourceLockArg {
    #[arg(long, default_value = parity::DEFAULT_SOURCE_LOCK)]
    source_lock: PathBuf,
}

#[derive(Debug, Args)]
struct ValidateArgs {
    #[command(flatten)]
    source: SourceLockArg,
    /// Root holding one checkout per lock repository (`<root>/<id>`, as
    /// materialized by `parity sync`). Enables deep validation of every
    /// repository row and the vendored `OpenAPI` body.
    #[arg(long, env = "XTASK_UPSTREAM_ROOT")]
    upstream_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct PolicyCli {
    #[command(subcommand)]
    command: PolicyCommand,
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    /// Run every repository-state policy check and summarize failures.
    All,
    /// Verify every public enum is classified in the enum policy manifest.
    #[command(name = "check-enum-policy")]
    CheckEnumPolicy(check_enum_policy::Args),
    /// Verify panic-bearing production source calls are explicitly allowlisted.
    #[command(name = "check-panic-allowlist")]
    CheckPanicAllowlist(check_panic_allowlist::Args),
    /// Verify `serde(deny_unknown_fields)` only appears on allowlisted SDK-owned DTOs.
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
    CheckAlloyProviderInvariant(dependency_invariant::Args),
    /// Verify shipped crates do not depend on alloy-signer.
    #[command(name = "check-alloy-signer-invariant")]
    CheckAlloySignerInvariant(dependency_invariant::Args),
    /// Verify principles and accepted ADRs are mutually covered.
    #[command(name = "check-adr-coverage")]
    CheckAdrCoverage(check_adr_coverage::Args),
    /// Verify alloy-* workspace pins are internally consistent per family.
    #[command(name = "check-alloy-family-pins")]
    CheckAlloyFamilyPins(check_alloy_family_pins::Args),
    /// Verify patch-lane chain additions do not require source-lock refresh.
    #[command(name = "check-chain-patch-eligibility")]
    CheckChainPatchEligibility(check_chain_patch_eligibility::Args),
    /// Verify property registry citations resolve to real test functions.
    #[command(name = "check-property-citations")]
    CheckPropertyCitations(check_property_citations::Args),
    /// Verify wasm package boundary invariants.
    #[command(name = "check-wasm-invariant")]
    CheckWasmInvariant(check_wasm_invariant::Args),
    /// Verify source-level never-swap fences hold.
    #[command(name = "check-source-fences")]
    CheckSourceFences(fences::Args),
    /// Verify workflow action refs are SHA-pinned and `pull_request_target` is reviewed.
    #[command(name = "check-workflow-security")]
    CheckWorkflowSecurity(check_workflow_security::Args),
    /// Verify no shell scripts exist outside the allowed lanes.
    #[command(name = "check-shell-wrappers")]
    CheckShellWrappers(check_shell_wrappers::Args),
    /// Verify consumer crates render their README on docs.rs.
    #[command(name = "check-readme-include")]
    CheckReadmeInclude(check_readme_include::Args),
    /// Execute every deterministic non-live example binary.
    #[command(name = "run-deterministic-examples")]
    RunDeterministicExamples(run_deterministic_examples::Args),
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Parity(command) => run_parity(command),
        Commands::Policy(cli) => run_policy(cli),
        Commands::Docs(DocsCommand::Agree(args)) => agree::run(&args),
        Commands::Docs(DocsCommand::AuditIndex(args)) => audit_index::run(&args),
    }
}

fn run_parity(command: ParityCommand) -> Result<()> {
    match command {
        ParityCommand::Validate(args) => parity::validate(&parity::CliOptions {
            source_lock: args.source.source_lock,
            // An empty env value means "not configured", not "the cwd".
            upstream_root: args
                .upstream_root
                .filter(|root| !root.as_os_str().is_empty()),
        }),
        ParityCommand::VendorOpenapi(args) => vendor_openapi::run(&args),
        ParityCommand::OpenapiCoverage(args) => openapi_coverage::run(&args),
        ParityCommand::ConfirmDeployments(args) => confirm_deployments(&args),
        ParityCommand::Sync(args) => sync::sync(&args),
        ParityCommand::Drift(args) => run_drift(&args),
    }
}

/// Exit codes: `0` no drift, `1` drift reported, `2` the comparison itself
/// failed (unreachable pin, fetch failure) — a louder signal than any diff.
fn run_drift(args: &sync::DriftArgs) -> Result<()> {
    match sync::drift(args) {
        Ok(sync::DriftStatus::Clean) => Ok(()),
        Ok(sync::DriftStatus::Drifted) => std::process::exit(1),
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::exit(2);
        }
    }
}

fn run_policy(cli: PolicyCli) -> Result<()> {
    match cli.command {
        PolicyCommand::All => xtask::policy::run_all(),
        PolicyCommand::CheckEnumPolicy(args) => check_enum_policy::run(args),
        PolicyCommand::CheckPanicAllowlist(args) => check_panic_allowlist::run(args),
        PolicyCommand::CheckDenyUnknownFields(args) => check_deny_unknown_fields::run(args),
        PolicyCommand::CheckWorkspaceVersions(args) => check_workspace_versions::run(&args),
        PolicyCommand::CheckMsrvNotice(args) => check_msrv_notice::run(&args),
        PolicyCommand::CheckAlloyProviderInvariant(args) => {
            dependency_invariant::run(&dependency_invariant::ALLOY_PROVIDER, &args)
        }
        PolicyCommand::CheckAlloySignerInvariant(args) => {
            dependency_invariant::run(&dependency_invariant::ALLOY_SIGNER, &args)
        }
        PolicyCommand::CheckAdrCoverage(args) => check_adr_coverage::run(args),
        PolicyCommand::CheckAlloyFamilyPins(args) => check_alloy_family_pins::run(&args),
        PolicyCommand::CheckChainPatchEligibility(args) => check_chain_patch_eligibility::run(args),
        PolicyCommand::CheckPropertyCitations(args) => check_property_citations::run(args),
        PolicyCommand::CheckWasmInvariant(args) => check_wasm_invariant::run(args),
        PolicyCommand::CheckSourceFences(args) => fences::run(&args),
        PolicyCommand::CheckWorkflowSecurity(args) => check_workflow_security::run(&args),
        PolicyCommand::CheckShellWrappers(args) => check_shell_wrappers::run(&args),
        PolicyCommand::CheckReadmeInclude(args) => check_readme_include::run(&args),
        PolicyCommand::RunDeterministicExamples(args) => run_deterministic_examples::run(&args),
    }
}

/// Runs the deployment-registry presence probe and exits with its status code
/// (`0` all present, `1` at least one failure). The exit is explicit so a
/// release run fails closed on an absent deployment rather than returning a
/// zero status with a non-empty failure list.
fn confirm_deployments(args: &ConfirmDeploymentsArgs) -> Result<()> {
    match registry_confirm::run(&args.confirm) {
        Ok(report) => {
            match args.format {
                OutputFormat::Text => println!("{}", report.render_text()),
                OutputFormat::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .expect("registry-confirm report should serialize")
                ),
            }
            std::process::exit(report.exit_code());
        }
        Err(error) => {
            emit_command_error(args.format, "XT10001", &error.to_string());
            std::process::exit(1);
        }
    }
}

fn emit_command_error(format: OutputFormat, code: &str, message: &str) {
    match format {
        OutputFormat::Text => eprintln!("error {code}: {message}"),
        OutputFormat::Json => eprintln!(
            "{}",
            serde_json::to_string(&json!({
                "level": "error",
                "code": code,
                "message": message,
            }))
            .expect("error diagnostic should serialize")
        ),
    }
}
