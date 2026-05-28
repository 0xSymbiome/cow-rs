//! Audit-self-pinning subcommand.
//!
//! Walks every JSON fixture under `parity/fixtures/`, classifies each fixture
//! by its authority shape, and cross-checks each rejected fixture against the
//! documented allowlist at `parity/self-pinning-allowlist.yaml`. The subcommand
//! emits a per-fixture report and a summary line. In report-only mode the
//! subcommand exits zero regardless of findings; in blocking mode the
//! subcommand exits non-zero when at least one rejected fixture is not
//! allowlisted.
//!
//! The authority hierarchy and the rationale for each permit and reject class
//! live in `docs/adr/0012-alloy-sol-bindings-and-registry-authority.md` and
//! the current-state review record at
//! `docs/audit/contract-bindings-parity-audit.md`.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Args;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value as JsonValue;

/// CLI arguments for the audit-self-pinning subcommand.
#[derive(Args, Debug)]
pub struct AuditSelfPinningArgs {
    /// Root directory containing parity fixtures. The subcommand recursively
    /// walks every `*.json` file under this root.
    #[arg(long, default_value = "parity/fixtures")]
    pub fixtures_root: PathBuf,

    /// Path to the self-pinning allowlist. When absent, the subcommand
    /// behaves as if the allowlist were empty.
    #[arg(long, default_value = "parity/self-pinning-allowlist.yaml")]
    pub allowlist: PathBuf,

    /// When set, the subcommand exits non-zero if any rejected fixture is
    /// not covered by the allowlist. When unset, the subcommand emits the
    /// report and exits zero regardless of findings.
    #[arg(long, default_value_t = false)]
    pub blocking: bool,
}

/// Classification of a single fixture's authority shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureClass {
    /// Fixture has no `expected:` block at top level or inside any row; it is
    /// consumed as pure input by a Rust test that compares against
    /// test-derived values inline. No external authority required.
    InputOnly,
    /// Fixture cites a specification document (EIP, ERC, RFC) as the
    /// protocol law for its surface.
    SpecAnchored,
    /// Fixture or its rows carry a structured `source_refs:`,
    /// `upstream_provenance:`, `expected_from:`, or per-row `@source_ref:`
    /// block citing an external authority by repo commit and path.
    Attributed,
    /// Top-level `source:` field names an in-tree Rust code path as sole
    /// authority. Rejected unless allowlisted.
    RustSelfPin,
    /// Top-level `source:` field is a non-empty string that fails the
    /// structured `<repo>:<sha>:<path>` pattern and the spec-citation
    /// pattern. Rejected unless allowlisted.
    FreeFormProse,
    /// Fixture appears to carry computed expectation values but exposes no
    /// `source:`, `source_refs:`, `@source_ref:`, `upstream_provenance:`,
    /// or `expected_from:` field. Rejected unless allowlisted.
    Missing,
}

impl FixtureClass {
    pub fn is_reject(self) -> bool {
        matches!(self, Self::RustSelfPin | Self::FreeFormProse | Self::Missing)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::InputOnly => "input-only",
            Self::SpecAnchored => "spec-anchored",
            Self::Attributed => "attributed",
            Self::RustSelfPin => "rust-self-pin",
            Self::FreeFormProse => "free-form-prose",
            Self::Missing => "missing",
        }
    }
}

/// Top-level shape of `parity/self-pinning-allowlist.yaml`.
#[derive(Debug, Deserialize)]
struct Allowlist {
    schema_version: u32,
    #[serde(default)]
    grandfathered: Vec<AllowlistEntry>,
    #[serde(default)]
    in_flight_upgrade: Vec<AllowlistEntry>,
}

/// A single allowlist entry. The audit subcommand treats `grandfathered:` and
/// `in_flight_upgrade:` entries identically for gate purposes; the separation
/// is documentation-only.
#[derive(Debug, Deserialize, Clone)]
struct AllowlistEntry {
    path: String,
    class: String,
    #[serde(default)]
    justification: Option<String>,
    #[serde(default)]
    defer_reason: Option<String>,
    #[serde(default)]
    review_trigger: Option<String>,
    #[serde(default)]
    upgrade_to: Option<String>,
}

/// Per-fixture row in the audit report.
#[derive(Debug, Clone)]
struct ReportEntry {
    path: String,
    class: FixtureClass,
    allowlist_match: Option<AllowlistMatch>,
}

/// Which allowlist block matched, and the entry's documented reason.
#[derive(Debug, Clone)]
struct AllowlistMatch {
    block: AllowlistBlock,
    /// Class the allowlist declares the fixture covers. Compared against the
    /// audit subcommand's own classification to catch drift between the
    /// allowlist's documentation and the fixture's current shape.
    declared_class: String,
    /// One of `justification:` or `defer_reason:` from the allowlist entry.
    reason_summary: String,
    review_trigger: Option<String>,
    upgrade_to: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllowlistBlock {
    Grandfathered,
    InFlightUpgrade,
}

impl AllowlistBlock {
    fn as_str(self) -> &'static str {
        match self {
            Self::Grandfathered => "grandfathered",
            Self::InFlightUpgrade => "in-flight",
        }
    }
}

/// Entry point for the subcommand.
pub fn run(args: &AuditSelfPinningArgs) -> Result<()> {
    let fixtures = walk_fixtures(&args.fixtures_root)?;
    let allowlist = load_allowlist(&args.allowlist)?;

    let mut entries: Vec<ReportEntry> = Vec::with_capacity(fixtures.len());
    for path in &fixtures {
        let class = classify(path)
            .with_context(|| format!("failed to classify {}", path.display()))?;
        let relative = relativize(path);
        let allowlist_match = match_allowlist(&relative, &allowlist);
        entries.push(ReportEntry {
            path: relative,
            class,
            allowlist_match,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    print_report(&entries);

    let class_mismatches: Vec<&ReportEntry> = entries
        .iter()
        .filter(|entry| match (&entry.allowlist_match, detected_class_token(entry.class)) {
            (Some(matched), Some(token)) => matched.declared_class != token,
            _ => false,
        })
        .collect();

    if !class_mismatches.is_empty() {
        println!();
        println!(
            "warning: {} allowlist entry/entries declare a class that does not match the detected reject class:",
            class_mismatches.len()
        );
        for entry in &class_mismatches {
            if let Some(matched) = entry.allowlist_match.as_ref() {
                println!(
                    "  {} (detected={}, allowlist declares={})",
                    entry.path,
                    entry.class.as_str(),
                    matched.declared_class
                );
            }
        }
    }

    // Stale-allowlist info note: an entry declares a reject class for a
    // fixture that the gate currently classifies as a permit class. The
    // entry is not blocking the gate (the fixture would pass without the
    // allowlist), but it may no longer be needed; reviewers consult the
    // entry's `review_trigger` to decide.
    let stale_entries: Vec<&ReportEntry> = entries
        .iter()
        .filter(|entry| {
            entry.allowlist_match.is_some()
                && detected_class_token(entry.class).is_none()
        })
        .collect();

    if !stale_entries.is_empty() {
        println!();
        println!(
            "info: {} allowlist entry/entries declare a reject class for fixtures the gate currently classifies as a permit class; review when the `review_trigger` fires:",
            stale_entries.len()
        );
        for entry in &stale_entries {
            if let Some(matched) = entry.allowlist_match.as_ref() {
                let trigger = matched
                    .review_trigger
                    .as_deref()
                    .unwrap_or("(no review trigger)");
                println!(
                    "  {} (detected={}, allowlist declares={}, trigger={})",
                    entry.path,
                    entry.class.as_str(),
                    matched.declared_class,
                    trigger,
                );
            }
        }
    }

    let rejected_without_allowlist: Vec<&ReportEntry> = entries
        .iter()
        .filter(|entry| entry.class.is_reject() && entry.allowlist_match.is_none())
        .collect();

    let permitted = entries.len() - entries.iter().filter(|e| e.class.is_reject()).count();
    let allowlisted = entries
        .iter()
        .filter(|e| e.class.is_reject() && e.allowlist_match.is_some())
        .count();
    let rejected = rejected_without_allowlist.len();

    println!();
    if args.blocking {
        println!(
            "audit-self-pinning audited {} fixtures: {} permit, {} allowlisted, {} rejected (blocking)",
            entries.len(),
            permitted,
            allowlisted,
            rejected
        );
    } else {
        println!(
            "audit-self-pinning audited {} fixtures: {} permit, {} allowlisted, {} rejected (report-only)",
            entries.len(),
            permitted,
            allowlisted,
            rejected
        );
    }

    if args.blocking && !rejected_without_allowlist.is_empty() {
        for entry in &rejected_without_allowlist {
            eprintln!(
                "error: {} ({}) is not covered by the self-pinning allowlist",
                entry.path,
                entry.class.as_str()
            );
        }
        bail!(
            "audit-self-pinning rejected {} fixture(s) not on the allowlist",
            rejected_without_allowlist.len()
        );
    }

    Ok(())
}

/// Recursively walks `root` and returns every `*.json` file path, sorted.
fn walk_fixtures(root: &Path) -> Result<Vec<PathBuf>> {
    let mut acc = Vec::new();
    if !root.exists() {
        return Ok(acc);
    }
    walk_dir(root, &mut acc)?;
    acc.sort();
    Ok(acc)
}

fn walk_dir(dir: &Path, acc: &mut Vec<PathBuf>) -> Result<()> {
    let read = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?;
    for entry in read {
        let entry = entry.with_context(|| format!("failed to read directory entry under {}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", path.display()))?;
        if file_type.is_dir() {
            walk_dir(&path, acc)?;
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("json")
        {
            acc.push(path);
        }
    }
    Ok(())
}

/// Loads the allowlist file when present, returning `None` when absent. A
/// schema-version mismatch is a hard error so silent allowlist drift cannot
/// land.
fn load_allowlist(path: &Path) -> Result<Option<Allowlist>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read allowlist {}", path.display()))?;
    let parsed: Allowlist = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse allowlist {}", path.display()))?;
    if parsed.schema_version != 1 {
        bail!(
            "{} declares schema_version {}, expected 1",
            path.display(),
            parsed.schema_version
        );
    }
    Ok(Some(parsed))
}

fn match_allowlist(fixture_path: &str, allowlist: &Option<Allowlist>) -> Option<AllowlistMatch> {
    let allowlist = allowlist.as_ref()?;
    if let Some(entry) = allowlist
        .grandfathered
        .iter()
        .find(|entry| entry.path == fixture_path)
    {
        return Some(allowlist_match_from(entry, AllowlistBlock::Grandfathered));
    }
    if let Some(entry) = allowlist
        .in_flight_upgrade
        .iter()
        .find(|entry| entry.path == fixture_path)
    {
        return Some(allowlist_match_from(entry, AllowlistBlock::InFlightUpgrade));
    }
    None
}

fn allowlist_match_from(entry: &AllowlistEntry, block: AllowlistBlock) -> AllowlistMatch {
    let reason_summary = entry
        .defer_reason
        .clone()
        .or_else(|| entry.justification.as_ref().map(|j| summarize(j)))
        .unwrap_or_else(|| "(no documented reason)".to_string());
    AllowlistMatch {
        block,
        declared_class: entry.class.clone(),
        reason_summary,
        review_trigger: entry.review_trigger.clone(),
        upgrade_to: entry.upgrade_to.as_ref().map(|p| summarize(p)),
    }
}

/// Maps the audit subcommand's detected class to the documented string the
/// allowlist YAML uses (one of `RustSelfPin`, `FreeFormProse`, `Missing`).
/// Returns `None` for permit classes since the allowlist never covers them.
fn detected_class_token(class: FixtureClass) -> Option<&'static str> {
    match class {
        FixtureClass::RustSelfPin => Some("RustSelfPin"),
        FixtureClass::FreeFormProse => Some("FreeFormProse"),
        FixtureClass::Missing => Some("Missing"),
        FixtureClass::InputOnly
        | FixtureClass::SpecAnchored
        | FixtureClass::Attributed => None,
    }
}

/// Truncates a multi-line justification to the first non-empty line for the
/// summary column.
fn summarize(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(text)
        .to_string()
}

/// Classifies a single fixture by inspecting its JSON shape against the
/// detection rules documented in
/// `docs/adr/0012-alloy-sol-bindings-and-registry-authority.md`.
fn classify(path: &Path) -> Result<FixtureClass> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read fixture {}", path.display()))?;
    let value: JsonValue = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse fixture {} as JSON", path.display()))?;

    let top = match value.as_object() {
        Some(map) => map,
        None => {
            // Top-level non-object fixtures are arrays of inputs; treat as
            // input-only.
            return Ok(FixtureClass::InputOnly);
        }
    };

    if let Some(arr) = top.get("source_refs").and_then(JsonValue::as_array)
        && !arr.is_empty()
    {
        return Ok(FixtureClass::Attributed);
    }
    if top.contains_key("upstream_provenance") {
        return Ok(FixtureClass::Attributed);
    }
    if let Some(arr) = top.get("expected_from").and_then(JsonValue::as_array) {
        if has_external_witness(arr) {
            return Ok(FixtureClass::Attributed);
        }
        return Ok(FixtureClass::RustSelfPin);
    }
    if let Some(source) = top.get("source").and_then(JsonValue::as_str) {
        if is_rust_self_pin(source) {
            return Ok(FixtureClass::RustSelfPin);
        }
        return Ok(FixtureClass::FreeFormProse);
    }

    // Some fixtures carry `upstream_provenance:` per row (one structured
    // citation per case) rather than at top level. Treat the recursive form
    // as the same permit class.
    if has_recursive_key(&value, "upstream_provenance") {
        return Ok(FixtureClass::Attributed);
    }

    let row_refs = collect_row_source_refs(&value);
    if !row_refs.is_empty() {
        if row_refs.iter().any(|r| is_spec_anchor(r)) {
            return Ok(FixtureClass::SpecAnchored);
        }
        if row_refs.iter().any(|r| is_repo_sha_anchor(r)) {
            return Ok(FixtureClass::Attributed);
        }
        // Row source_refs exist but none match a structured shape — treat as
        // free-form prose so it lands in the report.
        return Ok(FixtureClass::FreeFormProse);
    }

    if looks_expectation_bearing(&value) {
        Ok(FixtureClass::Missing)
    } else {
        Ok(FixtureClass::InputOnly)
    }
}

/// An `expected_from` array carries an external witness when at least one
/// entry has an authority value that is not `rust-alloy-sol-types` or
/// another in-tree-Rust marker.
fn has_external_witness(entries: &[JsonValue]) -> bool {
    entries.iter().any(|entry| {
        entry
            .get("authority")
            .and_then(JsonValue::as_str)
            .is_some_and(|auth| !is_in_tree_rust_authority(auth))
    })
}

fn is_in_tree_rust_authority(authority: &str) -> bool {
    matches!(
        authority,
        "rust-alloy-sol-types"
            | "rust-alloy-primitives"
            | "rust-cow-sdk-internal"
    )
}

fn is_rust_self_pin(source: &str) -> bool {
    let trimmed = source.trim_start();
    trimmed.starts_with("alloy_sol_types")
        || trimmed.starts_with("alloy_primitives")
        || trimmed.starts_with("cow_sdk_")
        || trimmed.starts_with("cow-sdk-")
        || trimmed.starts_with("crate::")
        || trimmed.starts_with("super::")
        || trimmed.starts_with("self::")
}

fn is_spec_anchor(source_ref: &str) -> bool {
    let trimmed = source_ref.trim_start();
    trimmed.starts_with("EIP-")
        || trimmed.starts_with("EIP ")
        || trimmed.starts_with("ERC-")
        || trimmed.starts_with("ERC ")
        || trimmed.starts_with("RFC ")
        || trimmed.starts_with("RFC-")
        || trimmed.starts_with("eip:")
        || trimmed.starts_with("rfc:")
        || trimmed.starts_with("erc:")
}

fn is_repo_sha_anchor(source_ref: &str) -> bool {
    // Matches `<repo>:<40-hex-sha>:<path>` shape; the path tail is optional.
    static_pattern().is_match(source_ref)
}

fn static_pattern() -> &'static Regex {
    use std::sync::OnceLock;
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"^[a-zA-Z][a-zA-Z0-9_\-/]*:[0-9a-fA-F]{40}(:|$)")
            .expect("static-pattern regex compiles")
    })
}

/// Collects every `@source_ref` or `source_ref` string value found by walking
/// the fixture's JSON tree. Returns an empty vector when no such field is
/// present anywhere.
fn collect_row_source_refs(value: &JsonValue) -> Vec<String> {
    let mut acc = Vec::new();
    collect_source_refs_recursive(value, &mut acc);
    acc
}

fn collect_source_refs_recursive(value: &JsonValue, acc: &mut Vec<String>) {
    match value {
        JsonValue::Object(map) => {
            for (key, val) in map {
                if (key == "@source_ref" || key == "source_ref")
                    && let Some(s) = val.as_str()
                {
                    acc.push(s.to_string());
                }
                collect_source_refs_recursive(val, acc);
            }
        }
        JsonValue::Array(items) => {
            for item in items {
                collect_source_refs_recursive(item, acc);
            }
        }
        _ => {}
    }
}

/// Returns true when at least one row under a top-level `rows:` or `cases:`
/// array either (a) carries an `expected:` wrapper key or (b) carries a
/// direct key whose name is in the strict computed-field set.
///
/// The check is intentionally scoped to row-level keys so a generic input
/// field name nested deep inside a row payload (for example `callData`
/// inside a hook target) does not trip the heuristic. Fixtures whose
/// structural shape places expectations elsewhere should declare an
/// `expected_from:` block or accept an allowlist entry — both are explicit
/// signals rather than heuristics.
fn looks_expectation_bearing(value: &JsonValue) -> bool {
    let Some(top) = value.as_object() else {
        return false;
    };
    for row_key in ["rows", "cases"] {
        let Some(arr) = top.get(row_key).and_then(JsonValue::as_array) else {
            continue;
        };
        for row in arr {
            let Some(obj) = row.as_object() else {
                continue;
            };
            if obj.contains_key("expected") {
                return true;
            }
            if obj.keys().any(|k| is_strict_computed_field(k)) {
                return true;
            }
        }
    }
    false
}

/// Strict-by-design set of field names whose presence at the row level
/// indicates the fixture pins a computed protocol value. Each member is a
/// finished-product field name that a reviewer would recognize as a
/// protocol-derived output rather than an input parameter.
fn is_strict_computed_field(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "digest"
            | "signing_hash"
            | "domain_separator"
            | "order_struct_hash"
            | "struct_hash"
            | "type_hash"
            | "order_type_hash"
            | "call_type_hash"
            | "execute_hooks_type_hash"
            | "selector"
            | "merkle_root"
            | "factory_call_data"
            | "proxy_call_data"
            | "expected_revert"
            | "expected_decoded"
            | "expected_classification"
    )
}

/// Returns true when any object inside the JSON tree carries `field_name`
/// as a key. Used to detect attribution blocks such as `upstream_provenance:`
/// that some fixtures place per row rather than at the top level.
fn has_recursive_key(value: &JsonValue, field_name: &str) -> bool {
    match value {
        JsonValue::Object(map) => {
            if map.contains_key(field_name) {
                return true;
            }
            map.values().any(|v| has_recursive_key(v, field_name))
        }
        JsonValue::Array(items) => items.iter().any(|v| has_recursive_key(v, field_name)),
        _ => false,
    }
}

fn relativize(path: &Path) -> String {
    let path_str = path.display().to_string();
    let normalized = path_str.replace('\\', "/");
    // Trim a leading `./` if present.
    if let Some(stripped) = normalized.strip_prefix("./") {
        stripped.to_string()
    } else {
        normalized
    }
}

fn print_report(entries: &[ReportEntry]) {
    let class_width = entries
        .iter()
        .map(|entry| entry.class.as_str().len())
        .max()
        .unwrap_or(8)
        .max("class".len());
    let allowlist_width = entries
        .iter()
        .map(|entry| {
            entry
                .allowlist_match
                .as_ref()
                .map(|m| m.block.as_str().len())
                .unwrap_or(1)
        })
        .max()
        .unwrap_or(13)
        .max("allowlist".len());
    let path_width = entries
        .iter()
        .map(|entry| entry.path.len())
        .max()
        .unwrap_or(4)
        .max("path".len());

    println!(
        "{:<class_width$}  {:<allowlist_width$}  {:<path_width$}",
        "class",
        "allowlist",
        "path",
        class_width = class_width,
        allowlist_width = allowlist_width,
        path_width = path_width,
    );
    println!(
        "{}  {}  {}",
        "-".repeat(class_width),
        "-".repeat(allowlist_width),
        "-".repeat(path_width),
    );
    for entry in entries {
        let allowlist_marker = entry
            .allowlist_match
            .as_ref()
            .map(|m| m.block.as_str())
            .unwrap_or("-");
        println!(
            "{:<class_width$}  {:<allowlist_width$}  {:<path_width$}",
            entry.class.as_str(),
            allowlist_marker,
            entry.path,
            class_width = class_width,
            allowlist_width = allowlist_width,
            path_width = path_width,
        );
    }

    // Per-allowlist documented-reason summary, grouped by block.
    let mut by_block: BTreeMap<&'static str, Vec<&ReportEntry>> = BTreeMap::new();
    for entry in entries {
        if let Some(m) = entry.allowlist_match.as_ref() {
            by_block.entry(m.block.as_str()).or_default().push(entry);
        }
    }
    if !by_block.is_empty() {
        println!();
        println!("allowlist documented reasons:");
        for (block, block_entries) in &by_block {
            println!("  [{block}]");
            for entry in block_entries {
                if let Some(m) = entry.allowlist_match.as_ref() {
                    let trigger = m
                        .review_trigger
                        .as_deref()
                        .unwrap_or("(no review trigger)");
                    println!(
                        "    {} ({}) — {} — review trigger: {}",
                        entry.path,
                        entry.class.as_str(),
                        m.reason_summary,
                        trigger,
                    );
                    if let Some(plan) = m.upgrade_to.as_deref() {
                        println!("      upgrade_to: {plan}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_self_pin_prefixes_are_detected() {
        assert!(is_rust_self_pin("alloy_sol_types::SolStruct::eip712"));
        assert!(is_rust_self_pin("alloy_primitives::keccak256"));
        assert!(is_rust_self_pin("cow_sdk_contracts::RecoverableSignature"));
        assert!(is_rust_self_pin("cow-sdk-contracts something"));
        assert!(is_rust_self_pin("crate::module::function"));
        assert!(is_rust_self_pin("super::helper"));
        assert!(is_rust_self_pin("self::inner"));
        assert!(!is_rust_self_pin("EIP-712 typedDataHash"));
        assert!(!is_rust_self_pin("RFC 7231 section 7.1.1.1"));
        assert!(!is_rust_self_pin("cowprotocol/contracts:c94c595a..."));
    }

    #[test]
    fn spec_anchors_are_detected() {
        assert!(is_spec_anchor("EIP-712 typedDataHash"));
        assert!(is_spec_anchor("EIP 712"));
        assert!(is_spec_anchor("ERC-20 approve"));
        assert!(is_spec_anchor("RFC 7231 section 7.1.1.1"));
        assert!(is_spec_anchor("RFC 8785 section 3.2.3 (object key ordering)"));
        assert!(is_spec_anchor("eip:eip-20#approve"));
        assert!(!is_spec_anchor("cowprotocol/contracts:c94c..."));
        assert!(!is_spec_anchor("alloy_sol_types::SolStruct"));
    }

    #[test]
    fn repo_sha_anchors_match_the_documented_shape() {
        assert!(is_repo_sha_anchor(
            "cowprotocol/contracts:c94c595a791681cf8ba7495117dcde397b932885:src/contracts/libraries/GPv2Order.sol"
        ));
        assert!(is_repo_sha_anchor(
            "cowdao-grants/cow-shed:9e01a88e0010314ee1e4c1a822105897a87d3bda:src/COWShed.sol"
        ));
        assert!(is_repo_sha_anchor(
            "services:0720b9bc15138ecc362078f505d0e3ba1c7b9883"
        ));
        assert!(!is_repo_sha_anchor("EIP-712"));
        assert!(!is_repo_sha_anchor("cowprotocol/contracts:not-a-sha:path"));
        assert!(!is_repo_sha_anchor("alloy_sol_types"));
    }

    #[test]
    fn classify_returns_attributed_for_source_refs_block() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("attributed.json");
        fs::write(
            &path,
            r#"{
              "source_refs": [
                { "repo": "cowprotocol/contracts", "commit": "c94c595a791681cf8ba7495117dcde397b932885", "path": "src/contracts/libraries/GPv2Order.sol" }
              ],
              "cases": []
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::Attributed);
    }

    #[test]
    fn classify_returns_spec_anchored_for_rfc_row_source_ref() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("spec.json");
        fs::write(
            &path,
            r#"{
              "rows": [
                { "@source_ref": "RFC 7231 section 7.1.1.1", "expected": { "value": "Wed, 21 Oct 2026 07:28:00 GMT" } }
              ]
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::SpecAnchored);
    }

    #[test]
    fn classify_returns_rust_self_pin_for_alloy_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("self_pin.json");
        fs::write(
            &path,
            r#"{
              "source": "alloy_sol_types::SolStruct::eip712_signing_hash on GPv2 Order",
              "rows": []
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::RustSelfPin);
    }

    #[test]
    fn classify_returns_free_form_prose_for_unstructured_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("prose.json");
        fs::write(
            &path,
            r#"{
              "source": "ComposableCoW.hash(ConditionalOrderParams)",
              "rows": []
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::FreeFormProse);
    }

    #[test]
    fn classify_returns_input_only_for_pure_input_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("input.json");
        fs::write(
            &path,
            r#"{
              "sender": "0x0000000000000000000000000000000000000005",
              "placementError": "none"
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::InputOnly);
    }

    #[test]
    fn classify_returns_missing_for_row_level_digest_without_authority() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing_digest.json");
        fs::write(
            &path,
            r#"{
              "rows": [
                { "chain_id": 1, "digest": "0xe489e6d7ce9431d0131bb4bf6a5b2919ad6e8da96b6130ff3a93f3bc806eb952" }
              ]
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::Missing);
    }

    #[test]
    fn classify_returns_missing_for_row_level_expected_wrapper_without_authority() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing_expected.json");
        fs::write(
            &path,
            r#"{
              "rows": [
                { "inputs": { "r": "0x11", "s": "0x22", "v": 27 }, "expected": { "packed_signature": "0xabcd" } }
              ]
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::Missing);
    }

    #[test]
    fn classify_returns_input_only_for_nested_calldata_without_row_level_expectation() {
        // A hook-payload-shaped fixture: `callData` lives deep inside the
        // input data and is NOT an output expectation. The row-scoped
        // classifier must not trip on it.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("input_with_nested_calldata.json");
        fs::write(
            &path,
            r#"{
              "version": "1.14.0",
              "metadata": {
                "hooks": {
                  "pre": [
                    { "target": "0x1234567890abcdef1234567890abcdef12345678", "callData": "0x01020304" }
                  ]
                }
              }
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::InputOnly);
    }

    #[test]
    fn classify_returns_attributed_for_per_row_upstream_provenance() {
        // The TWAP fixtures carry `upstream_provenance:` per row rather
        // than at top level. The recursive detection must still classify
        // the fixture as attributed.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("per_row_provenance.json");
        fs::write(
            &path,
            r#"{
              "rows": [
                {
                  "inputs": { "params_hash": "0xbd9e..." },
                  "expected": { "merkle_leaf": "0xb336..." },
                  "upstream_provenance": [
                    { "repo": "composable-cow", "path": "src/ComposableCoW.sol" }
                  ]
                }
              ]
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::Attributed);
    }

    #[test]
    fn classify_recognizes_expected_from_block_with_external_witness() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("expected_from.json");
        fs::write(
            &path,
            r#"{
              "schema_version": 2,
              "expected_from": [
                { "authority": "rust-alloy-sol-types", "implementation": "alloy_sol_types" },
                { "authority": "solidity-eip712-test-library", "repo": "cowprotocol/contracts", "commit": "c94c595a791681cf8ba7495117dcde397b932885", "path": "test/libraries/Eip712.sol" }
              ],
              "rows": []
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::Attributed);
    }

    #[test]
    fn classify_returns_rust_self_pin_for_expected_from_without_external_witness() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("self_pin_array.json");
        fs::write(
            &path,
            r#"{
              "expected_from": [
                { "authority": "rust-alloy-sol-types" }
              ],
              "rows": []
            }"#,
        )
        .expect("write");
        assert_eq!(classify(&path).expect("classify"), FixtureClass::RustSelfPin);
    }
}
