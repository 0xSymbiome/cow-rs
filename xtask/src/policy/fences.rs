//! Source-level "never-swap" fences: lexical bans that keep alloy-adjacent
//! hand-rolls and forbidden imports out of protected crate surfaces.
//!
//! Each [`Fence`] is one constraint anchored by an ADR (see
//! `docs/alloy-doctrine.md`). A fence scans a scope of files for a forbidden
//! pattern, or asserts a required marker count, and fails with the ADR-cited
//! guidance. This table is the Rust home of the former `never-swap-gates.yml`
//! grep jobs plus the `encode_prefixed`, forbidden-wasm-import, and dist-path
//! gates: one reviewable place for the patterns, their scopes, and their
//! messages, each with unit-test fixtures.
//!
//! Matching is line-based and replicates the source greps exactly, including
//! the `//`-comment skip that keeps the explanatory `// DO NOT SWAP` blocks
//! from self-triggering.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::policy::workspace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

/// The files a fence scans.
enum Scope {
    /// All `*.rs` recursively under these directories.
    RustUnder(&'static [&'static str]),
    /// These exact files.
    Files(&'static [&'static str]),
    /// All `*.rs` under `crates/`, excluding crate-level
    /// `tests`/`benches`/`examples`/`fuzz` directories.
    ProductionRust,
    /// All `*.rs` under `crates/wasm/src`.
    WasmSources,
    /// Documentation and TypeScript surfaces that must not deep-import dist paths.
    DocsAndTypeScript,
}

/// What a fence asserts about the lines it scans.
enum Rule {
    /// Fail on any line matching `pattern`. When `skip_line_comments`, lines
    /// whose first non-space characters are `//` are ignored.
    Forbid {
        pattern: &'static str,
        skip_line_comments: bool,
    },
    /// Fail unless exactly `expected` lines contain `needle`.
    Count {
        needle: &'static str,
        expected: usize,
    },
}

/// Whether an empty candidate set passes or fails closed.
enum OnEmpty {
    /// No candidates is fine — the protected surface may not exist yet.
    Pass,
    /// No candidates is a failure — refuse to silently pass.
    Fail,
}

struct Fence {
    name: &'static str,
    scope: Scope,
    rule: Rule,
    on_empty: OnEmpty,
    message: &'static str,
}

const FENCES: &[Fence] = &[
    Fence {
        name: "ecdsa-v-normalization",
        scope: Scope::RustUnder(&["crates/contracts/src", "crates/signing/src"]),
        rule: Rule::Forbid {
            pattern: r"\bnormalize_v\b|\balloy_primitives::Signature::v\b|\b(AlloySignature|alloy_primitives::Signature)::from_raw\b|\bSignature::as_rsy\b",
            skip_line_comments: true,
        },
        on_empty: OnEmpty::Pass,
        message: "cow-rs emits v in {27, 28} for on-chain ecrecover (ADR 0022). Reroute through cow_sdk_contracts::RecoverableSignature; direct alloy parity-normalization, Signature::from_raw, or Signature::as_rsy widens the input or output surface beyond the canonical contract.",
    },
    Fence {
        name: "amount-radix",
        scope: Scope::Files(&["crates/core/src/types/amount.rs"]),
        rule: Rule::Forbid {
            pattern: r"\b(U256|I256)::from_str\s*\(",
            skip_line_comments: true,
        },
        on_empty: OnEmpty::Fail,
        message: "alloy U256 from_str sniffs 0x/0o/0b radix prefixes. cow Amount enforces explicit radix via from_str_radix and from_dec_str (ADR 0052).",
    },
    Fence {
        name: "address-display-lowercase",
        scope: Scope::Files(&["crates/core/src/types/identity.rs"]),
        rule: Rule::Forbid {
            pattern: r"#\[derive\([^)]*Display[^)]*\)\]|\bto_checksum(_buffer)?\s*\(|\.to_checksum\b",
            skip_line_comments: true,
        },
        on_empty: OnEmpty::Fail,
        message: "cow Address wire form is lowercase; alloy default Display is EIP-55 checksum (ADR 0052). Hand-write Display via {:#x}; do not derive.",
    },
    Fence {
        name: "typed-data-domain-dto-field",
        scope: Scope::RustUnder(&[
            "crates/core/src/traits",
            "crates/core/src/types",
            "crates/signing/src",
        ]),
        rule: Rule::Forbid {
            pattern: r"^\s*pub\s+\w+\s*:\s*(alloy_sol_types::)?Eip712Domain\b",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "alloy Eip712Domain is the hashing-side type with Option fields, U256 chainId, and salt; cow TypedDataDomain emits the EIP-1193 eth_signTypedData_v4 wire shape (ADR 0052, ADR 0040). Use TypedDataDomain on DTO fields; bridge through .into_alloy_domain() for hashing.",
    },
    Fence {
        name: "eip1271-shape-flag",
        scope: Scope::RustUnder(&["crates/signing/src/eip1271"]),
        rule: Rule::Forbid {
            pattern: r"(shape|kind|variant)\s*:\s*(Shape|BlobShape|Eip1271(Blob)?Shape)\b|fn\s+encode_eip1271_blob\s*<|fn\s+encode_blob_(any|either|both)\b",
            skip_line_comments: true,
        },
        on_empty: OnEmpty::Pass,
        message: "ADR 0050 requires Shape A (Safe muxer, selector-prefixed) and Shape B (raw forwarder, no selector) to be distinct encoder entry points. Do not pass a shape: ShapeKind flag.",
    },
    Fence {
        name: "rest-transport-stack",
        scope: Scope::RustUnder(&[
            "crates/core/src/transport",
            "crates/transport-wasm/src",
            "crates/orderbook/src",
            "crates/subgraph/src",
        ]),
        rule: Rule::Forbid {
            pattern: r"\balloy_transport(_http)?\b|\bRequestPacket\b|\bResponsePacket\b|alloy[_-]json[_-]rpc::|\bRetryBackoffLayer\b",
            skip_line_comments: true,
        },
        on_empty: OnEmpty::Fail,
        message: "alloy transport is tower::Service<RequestPacket> over JSON-RPC; cow-rs REST transport is HttpTransport over arbitrary JSON bodies (ADR 0010, 0019, 0041, 0046). REST crates must not import alloy-transport.",
    },
    Fence {
        name: "do-not-swap-census",
        scope: Scope::RustUnder(&["crates"]),
        rule: Rule::Count {
            needle: "DO NOT SWAP",
            expected: 10,
        },
        on_empty: OnEmpty::Fail,
        message: "expected exactly 10 DO NOT SWAP comment blocks across crates/ (one per anchor; one anchor carries paired blocks). Restore the missing comment, or update this fence's expected count and amend docs/alloy-doctrine.md.",
    },
    Fence {
        name: "encode-prefixed-hand-roll",
        scope: Scope::ProductionRust,
        rule: Rule::Forbid {
            pattern: r#"\bformat!\s*\(\s*"0x\{\}"\s*,\s*(alloy_primitives::)?hex::encode(_upper)?\s*\("#,
            skip_line_comments: true,
        },
        on_empty: OnEmpty::Fail,
        message: "format!(\"0x{}\", alloy_primitives::hex::encode(...)) is the legacy hand-roll; alloy_primitives::hex::encode_prefixed emits the same 0x-prefixed lowercase string in one call (ADR 0052).",
    },
    Fence {
        name: "encode-prefixed-unqualified-import",
        scope: Scope::ProductionRust,
        rule: Rule::Forbid {
            pattern: r"^\s*use\s+alloy_primitives::hex::encode(\s*;|\s+as\s)",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Fail,
        message: "importing alloy_primitives::hex::encode unqualified into production sources enables the legacy format!(\"0x{}\", encode(...)) hand-roll. Use alloy_primitives::hex::encode_prefixed (ADR 0052).",
    },
    Fence {
        name: "wasm-no-alloy-family",
        scope: Scope::WasmSources,
        rule: Rule::Forbid {
            pattern: r"cow[-_]sdk[-_]alloy([-_](provider|signer))?",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "cow-sdk-wasm must not reference the native alloy adapter crates.",
    },
    Fence {
        name: "wasm-no-reqwest",
        scope: Scope::WasmSources,
        rule: Rule::Forbid {
            pattern: r"\breqwest\b",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "cow-sdk-wasm must not reference reqwest; the browser transport is the Fetch seam.",
    },
    Fence {
        name: "wasm-no-tokio-runtime",
        scope: Scope::WasmSources,
        rule: Rule::Forbid {
            pattern: r"tokio::(spawn|runtime|time)",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "cow-sdk-wasm must not reference tokio runtime parts.",
    },
    Fence {
        name: "wasm-no-tokio-macros",
        scope: Scope::WasmSources,
        rule: Rule::Forbid {
            pattern: r"tokio::(macros)",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "cow-sdk-wasm must not reference tokio macros.",
    },
    Fence {
        name: "wasm-no-core-reqwest-reexports",
        scope: Scope::WasmSources,
        rule: Rule::Forbid {
            pattern: r"cow_sdk_core::Reqwest(Transport|TransportConfig|classify_reqwest_error)",
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "cow-sdk-wasm must not reference the cow-sdk-core reqwest re-exports.",
    },
    Fence {
        name: "wasm-no-dist-deep-import",
        scope: Scope::DocsAndTypeScript,
        rule: Rule::Forbid {
            pattern: r#"cow-sdk-wasm(?:-test-package)?(?:/[^"'`\s\\]+)?/dist/"#,
            skip_line_comments: false,
        },
        on_empty: OnEmpty::Pass,
        message: "public imports must use package subpaths, not dist paths.",
    },
];

pub fn run_default() -> Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> Result<()> {
    let mut failures = Vec::new();
    for fence in FENCES {
        match evaluate(&args.repo_root, fence) {
            Ok(violations) => {
                failures.extend(violations.into_iter().map(|v| format!("[{}] {v}", fence.name)));
            }
            Err(error) => failures.push(format!("[{}] {error:#}", fence.name)),
        }
    }

    if failures.is_empty() {
        println!("source fences hold ({} fences)", FENCES.len());
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("error: {failure}");
        }
        bail!("{} source-fence violation(s)", failures.len())
    }
}

fn evaluate(repo_root: &Path, fence: &Fence) -> Result<Vec<String>> {
    let files = collect(repo_root, &fence.scope)?;
    if files.is_empty() {
        return match fence.on_empty {
            OnEmpty::Pass => Ok(Vec::new()),
            OnEmpty::Fail => {
                Ok(vec!["no candidate files under scope; refusing to silently pass".to_owned()])
            }
        };
    }

    match &fence.rule {
        Rule::Forbid {
            pattern,
            skip_line_comments,
        } => forbid(repo_root, &files, pattern, *skip_line_comments, fence.message),
        Rule::Count { needle, expected } => count(&files, needle, *expected, fence.message),
    }
}

fn forbid(
    repo_root: &Path,
    files: &[PathBuf],
    pattern: &str,
    skip_line_comments: bool,
    message: &str,
) -> Result<Vec<String>> {
    let regex = Regex::new(pattern).with_context(|| format!("invalid fence pattern `{pattern}`"))?;
    let mut violations = Vec::new();
    for file in files {
        let text = read(file)?;
        for (index, line) in text.lines().enumerate() {
            if skip_line_comments && line.trim_start().starts_with("//") {
                continue;
            }
            if regex.is_match(line) {
                violations.push(format!(
                    "{}:{}: {} -- {message}",
                    workspace::relative_path(repo_root, file),
                    index + 1,
                    line.trim(),
                ));
            }
        }
    }
    Ok(violations)
}

fn count(files: &[PathBuf], needle: &str, expected: usize, message: &str) -> Result<Vec<String>> {
    let mut total = 0usize;
    for file in files {
        total += read(file)?.lines().filter(|line| line.contains(needle)).count();
    }
    if total == expected {
        Ok(Vec::new())
    } else {
        Ok(vec![format!(
            "found {total} line(s) containing `{needle}`, expected {expected} -- {message}"
        )])
    }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn collect(repo_root: &Path, scope: &Scope) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    match scope {
        Scope::RustUnder(dirs) => {
            for dir in *dirs {
                collect_by_extension(&repo_root.join(dir), "rs", &mut files)?;
            }
        }
        Scope::Files(paths) => {
            for path in *paths {
                let file = repo_root.join(path);
                if file.is_file() {
                    files.push(file);
                }
            }
        }
        Scope::ProductionRust => collect_production_rust(repo_root, &mut files)?,
        Scope::WasmSources => collect_by_extension(&repo_root.join("crates/wasm/src"), "rs", &mut files)?,
        Scope::DocsAndTypeScript => collect_docs_and_typescript(repo_root, &mut files)?,
    }
    files.sort();
    Ok(files)
}

const DOC_TS_EXTENSIONS: &[&str] = &["md", "toml", "ts", "tsx", "js", "mjs", "json", "jsonc"];
const PRODUCTION_EXCLUDED: &[&str] = &["tests", "benches", "examples", "fuzz"];

fn collect_by_extension(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    walk(dir, files, &mut |path| {
        path.extension().is_some_and(|ext| ext == extension)
    })
}

/// All `*.rs` under `crates/`, excluding crate-level
/// `tests`/`benches`/`examples`/`fuzz` directories (the `crates/*/<dir>/`
/// shape the source grep skips with `-not -path 'crates/*/<dir>/*'`).
fn collect_production_rust(repo_root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let crates = repo_root.join("crates");
    walk(&crates, files, &mut |path| {
        if path.extension().is_none_or(|ext| ext != "rs") {
            return false;
        }
        let Ok(relative) = path.strip_prefix(&crates) else {
            return true;
        };
        // relative = <crate>/<second>/...; exclude when <second> is a guarded dir.
        let second = relative
            .components()
            .nth(1)
            .and_then(|component| match component {
                Component::Normal(part) => part.to_str(),
                _ => None,
            });
        !second.is_some_and(|part| PRODUCTION_EXCLUDED.contains(&part))
    })
}

fn collect_docs_and_typescript(repo_root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let matches = |path: &Path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| DOC_TS_EXTENSIONS.contains(&ext))
    };
    push_if_file_matching(repo_root.join("README.md"), &matches, files);
    push_if_file_matching(repo_root.join("CONTRIBUTING.md"), &matches, files);
    for root in ["docs", "crates", "e2e", "examples"] {
        walk(&repo_root.join(root), files, &mut |path| {
            if path.components().any(|component| {
                matches!(component, Component::Normal(part) if part == "node_modules" || part == "target")
            }) {
                return false;
            }
            matches(path)
        })?;
    }
    Ok(())
}

fn push_if_file_matching(path: PathBuf, matches: &impl Fn(&Path) -> bool, files: &mut Vec<PathBuf>) {
    if path.is_file() && matches(&path) {
        files.push(path);
    }
}

/// Recursively collects files under `dir` for which `accept` returns true.
/// A missing directory is skipped rather than an error.
fn walk(dir: &Path, files: &mut Vec<PathBuf>, accept: &mut impl FnMut(&Path) -> bool) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            walk(&path, files, accept)?;
        } else if accept(&path) {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FENCES, Rule};
    use regex::Regex;

    #[test]
    fn every_forbid_pattern_compiles() {
        for fence in FENCES {
            if let Rule::Forbid { pattern, .. } = &fence.rule {
                Regex::new(pattern)
                    .unwrap_or_else(|error| panic!("fence `{}` pattern is invalid: {error}", fence.name));
            }
        }
    }

    #[test]
    fn ecdsa_fence_trips_on_normalize_v_but_not_on_a_comment() {
        let fence = FENCES
            .iter()
            .find(|fence| fence.name == "ecdsa-v-normalization")
            .expect("ecdsa fence is registered");
        let Rule::Forbid { pattern, skip_line_comments } = &fence.rule else {
            panic!("ecdsa fence is a forbid rule");
        };
        let regex = Regex::new(pattern).unwrap();
        assert!(*skip_line_comments);

        let offending = "let v = normalize_v(raw);";
        assert!(regex.is_match(offending));

        // A `// DO NOT SWAP` comment naming the symbol must be skipped.
        let comment = "// DO NOT SWAP: normalize_v admits EIP-155 v >= 35.";
        assert!(regex.is_match(comment));
        assert!(comment.trim_start().starts_with("//"));
    }

    #[test]
    fn encode_prefixed_fence_matches_the_legacy_hand_roll() {
        let fence = FENCES
            .iter()
            .find(|fence| fence.name == "encode-prefixed-hand-roll")
            .expect("encode-prefixed fence is registered");
        let Rule::Forbid { pattern, .. } = &fence.rule else {
            panic!("encode-prefixed fence is a forbid rule");
        };
        let regex = Regex::new(pattern).unwrap();
        assert!(regex.is_match(r#"format!("0x{}", hex::encode(bytes))"#));
        assert!(regex.is_match(r#"format!("0x{}", alloy_primitives::hex::encode_upper(bytes))"#));
        assert!(!regex.is_match("alloy_primitives::hex::encode_prefixed(bytes)"));
    }
}
