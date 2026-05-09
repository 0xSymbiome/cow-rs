use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};

use crate::diagnostics::{Diagnostic, OutputMode};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let repo_root = args.repo_root;
    run_host_gate(&repo_root)?;

    let mut failures = Vec::new();
    check_source_invariants(&repo_root, &mut failures)?;

    if failures.is_empty() {
        Diagnostic::info("PM9000", "wasm invariants hold").emit(output_mode, writer)?;
        return Ok(());
    }

    let detail = failures.join("\n");
    Diagnostic::error("PM9001", &detail).emit(output_mode, writer)?;
    bail!("{detail}")
}

fn run_host_gate(repo_root: &Path) -> anyhow::Result<()> {
    let output = Command::new("cargo")
        .current_dir(repo_root)
        .args(["check", "-p", "cow-sdk-wasm", "--no-default-features"])
        .output()
        .context("failed to invoke cargo check for cow-sdk-wasm")?;

    if output.status.success() {
        return Ok(());
    }

    bail!(
        "cow-sdk-wasm host gate failed; keep wasm-bindgen ABI derives out of pure helpers\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn check_source_invariants(repo_root: &Path, failures: &mut Vec<String>) -> anyhow::Result<()> {
    let pure_mod = read(repo_root, "crates/wasm/src/pure/mod.rs")?;
    let exports_mod = read(repo_root, "crates/wasm/src/exports/mod.rs")?;
    let transport = read(repo_root, "crates/wasm/src/exports/transport.rs")?;
    let errors = read(repo_root, "crates/wasm/src/exports/errors.rs")?;

    require_contains(
        &pure_mod,
        "pub mod",
        "pure helper modules must remain host-visible",
        failures,
    );
    require_contains(
        &exports_mod,
        "pub mod",
        "wasm export modules must remain isolated under exports",
        failures,
    );
    require_contains(
        &transport,
        "js_sys::Promise::resolve",
        "callback transport must normalize callback returns with Promise.resolve",
        failures,
    );
    reject_contains(
        &transport,
        "Promise::from",
        "callback transport must not use Promise::from; use Promise.resolve",
        failures,
    );
    require_contains(
        &transport,
        "handle: Option<JsValue>",
        "timer handle must stay opaque so numeric and object handles both clear correctly",
        failures,
    );
    require_contains(
        &transport,
        "impl Drop for TimerGuard",
        "TimerGuard must keep a Drop cleanup path",
        failures,
    );
    require_contains(
        &transport,
        "Reflect::set(&dto, &\"signal\".into(), signal.as_ref())",
        "callback request DTO must carry an AbortSignal",
        failures,
    );
    reject_contains(
        &transport,
        "Closure::forget",
        "callback transport must retain and drop timeout closures",
        failures,
    );
    reject_contains(
        &transport,
        ".forget()",
        "callback transport must not leak closures",
        failures,
    );
    reject_contains(
        &errors,
        "Redacted::into_inner",
        "WasmError mapping must preserve redacted values",
        failures,
    );
    reject_contains(
        &errors,
        ".into_inner()",
        "WasmError mapping must not unwrap redacted values",
        failures,
    );

    Ok(())
}

fn read(repo_root: &Path, relative: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(repo_root.join(relative))
        .with_context(|| format!("failed to read {relative}"))
}

fn require_contains(source: &str, needle: &str, message: &str, failures: &mut Vec<String>) {
    if !source.contains(needle) {
        failures.push(format!("{message}; expected to find `{needle}`"));
    }
}

fn reject_contains(source: &str, needle: &str, message: &str, failures: &mut Vec<String>) {
    if source.contains(needle) {
        failures.push(format!("{message}; remove `{needle}`"));
    }
}
