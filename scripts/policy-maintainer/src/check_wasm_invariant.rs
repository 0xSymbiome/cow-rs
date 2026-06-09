use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};

use crate::diagnostics::{Diagnostic, OutputMode};

const WASM_OPT_FLAGS: &[&str] = &[
    "-Oz",
    "--enable-bulk-memory",
    "--enable-sign-ext",
    "--strip-debug",
    "--strip-producers",
    "--vacuum",
    "--merge-blocks",
    "--simplify-locals",
    "--enable-nontrapping-float-to-int",
    "--enable-simd",
];

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
    let helpers_mod = read(repo_root, "crates/wasm/src/helpers/mod.rs")?;
    let exports_mod = read(repo_root, "crates/wasm/src/exports/mod.rs")?;
    let transport = read(repo_root, "crates/wasm/src/exports/transport.rs")?;
    let errors = read(repo_root, "crates/wasm/src/exports/errors.rs")?;

    require_contains(
        &helpers_mod,
        "pub mod",
        "host-safe helper modules must remain host-visible under src/helpers",
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
    check_build_pipeline_invariants(repo_root, failures)?;

    Ok(())
}

fn check_build_pipeline_invariants(
    repo_root: &Path,
    failures: &mut Vec<String>,
) -> anyhow::Result<()> {
    let build_script = read(repo_root, "crates/wasm/npm/scripts/build.sh")?;
    let crate_manifest = read(repo_root, "crates/wasm/Cargo.toml")?;
    require_contains(
        &build_script,
        "wasm-pack build",
        "npm build script must invoke wasm-pack",
        failures,
    );
    require_wasm_opt_after_every_wasm_pack(&build_script, failures);
    for flag in WASM_OPT_FLAGS {
        require_contains(
            &build_script,
            flag,
            "npm build script must keep the required wasm-opt flag set",
            failures,
        );
    }
    require_contains(
        &crate_manifest,
        "[package.metadata.wasm-pack.profile.release]",
        "cow-sdk-wasm must disable wasm-pack implicit release optimization",
        failures,
    );
    require_contains(
        &crate_manifest,
        "wasm-opt = false",
        "cow-sdk-wasm must leave release optimization to the explicit build post-pass",
        failures,
    );

    let scripts_dir = repo_root.join("crates/wasm/npm/scripts");
    if scripts_dir.join("build.ps1").exists() {
        failures.push("PowerShell wasm build entrypoint must not be added".to_string());
    }
    if scripts_dir.is_dir() {
        for entry in fs::read_dir(&scripts_dir)
            .with_context(|| format!("failed to read {}", scripts_dir.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read {}", scripts_dir.display()))?;
            let path = entry.path();
            if has_extension(&path, "ps1") {
                failures.push(format!(
                    "PowerShell build script is out of policy: {}",
                    path.display()
                ));
            }
        }
    }

    Ok(())
}

fn require_wasm_opt_after_every_wasm_pack(source: &str, failures: &mut Vec<String>) {
    let invocations = source
        .match_indices("wasm-pack build")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    if invocations.is_empty() {
        failures.push("npm build script must invoke wasm-pack build".to_string());
        return;
    }

    for (slot, start) in invocations.iter().copied().enumerate() {
        let end = invocations
            .get(slot + 1)
            .copied()
            .unwrap_or(source.len());
        let block = &source[start..end];
        if !block.contains("wasm-opt") && !block.contains("optimize_wasm_output") {
            failures.push(
                "npm build script must run wasm-opt after every wasm-pack build invocation"
                    .to_string(),
            );
        }
    }
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(expected))
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

#[cfg(test)]
mod tests {
    use super::require_wasm_opt_after_every_wasm_pack;

    #[test]
    fn wasm_opt_must_follow_each_wasm_pack_invocation() {
        let mut failures = Vec::new();
        require_wasm_opt_after_every_wasm_pack(
            r#"
            wasm-pack build crates/wasm --target web --release
            wasm-opt -Oz out.wasm -o out.opt.wasm
            wasm-pack build crates/wasm --target nodejs --release
            "#,
            &mut failures,
        );

        assert_eq!(
            failures,
            ["npm build script must run wasm-opt after every wasm-pack build invocation"]
        );
    }
}
