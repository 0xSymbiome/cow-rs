//! `check-audit-lane` — the audit-lane analogue of `check-principles`.
//!
//! Validates every `docs/audit/*-audit.md`:
//!
//! - OKF frontmatter (`type: Audit`, `id`, `title`, `description`, `status`,
//!   `owning_surface`, `refresh_trigger`, `related`, `timestamp`) is present and
//!   well-formed;
//! - `description` is a hand-written sentence, not machine-truncated junk;
//! - the body uses the fixed `Scope` / `Findings` / `Evidence` skeleton (with a
//!   `Tracked advisories` exception for the dependency-gate audit);
//! - `related` lists only resolvable `ADR-NNNN` identifiers; and
//! - the ADR↔audit `**Proven by:**` ↔ `related` graph is reciprocal in both
//!   directions.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    // `timestamp` (a YAML date) is intentionally omitted from this struct and
    // checked by line scan instead, so date deserialization cannot fail the
    // parse. The audit-index gate validates its format and index agreement.
    #[serde(rename = "type")]
    kind: Option<String>,
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    owning_surface: Option<String>,
    #[serde(default)]
    related: Vec<String>,
}

/// `.github/config/audit-refresh-map.yml`: the single machine-readable source of
/// each audit's owning surface and refresh triggers (the per-file prose
/// `refresh_trigger` was retired in favour of this map).
#[derive(Debug, Deserialize)]
pub struct RefreshMap {
    pub entries: Vec<RefreshEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshEntry {
    pub audit: String,
    pub owning_surface: String,
    pub refresh_triggers: RefreshTriggers,
}

#[derive(Debug, Default, Deserialize)]
pub struct RefreshTriggers {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub upstreams: Vec<String>,
}

/// The audit slug for a map entry whose `audit` path is well-formed.
pub fn entry_slug(entry: &RefreshEntry) -> Option<&str> {
    entry
        .audit
        .strip_prefix("docs/audit/")
        .and_then(|rest| rest.strip_suffix("-audit.md"))
}

/// Loads and parses `.github/config/audit-refresh-map.yml`.
pub fn load_refresh_map(root: &Path) -> Result<RefreshMap> {
    let path = root.join(".github/config/audit-refresh-map.yml");
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_norway::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

const ALLOWED_STATUS: [&str; 3] = ["Current", "Refresh required", "Superseded"];

/// Entry point for the `cargo check-policies` aggregator.
pub fn run_default() -> Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> Result<()> {
    let root = &args.repo_root;
    let audit_dir = root.join("docs/audit");
    let adr_dir = root.join("docs/adr");

    let mut errors = Vec::new();

    // audit slug -> the set of `ADR-NNNN` it declares in `related`.
    let mut audit_related: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // audit slug -> its frontmatter `owning_surface` (checked against the map).
    let mut audit_owning: BTreeMap<String, String> = BTreeMap::new();

    let mut audit_files = collect_md(&audit_dir, &["index.md"])
        .with_context(|| format!("failed to read {}", audit_dir.display()))?;
    audit_files.sort();

    for path in &audit_files {
        let name = file_name(path);
        let slug = name.strip_suffix("-audit.md").map(str::to_owned);
        let rel = format!("docs/audit/{name}");
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let Some((front_text, body)) = split_frontmatter(&text) else {
            errors.push(format!("::error file={rel}::missing frontmatter block"));
            continue;
        };
        let front: Frontmatter = match serde_norway::from_str(front_text) {
            Ok(front) => front,
            Err(error) => {
                errors.push(format!(
                    "::error file={rel}::frontmatter does not parse: {error}"
                ));
                continue;
            }
        };

        validate_frontmatter(&mut errors, &rel, slug.as_deref(), &front, front_text);
        validate_sections(&mut errors, &rel, slug.as_deref(), body);

        let Some(slug) = slug else { continue };
        audit_owning.insert(
            slug.clone(),
            front.owning_surface.clone().unwrap_or_default(),
        );
        let mut set = BTreeSet::new();
        for token in &front.related {
            if !is_adr_id(token) {
                errors.push(format!(
                    "::error file={rel}::`related` entry {token:?} is not an ADR-NNNN identifier"
                ));
                continue;
            }
            if !adr_exists(&adr_dir, token) {
                errors.push(format!(
                    "::error file={rel}::`related` ADR {token} does not resolve to a docs/adr file"
                ));
                continue;
            }
            set.insert(token.clone());
        }
        audit_related.insert(slug, set);
    }

    // `ADR-NNNN` -> the set of audit slugs under its `**Proven by:**` block.
    let adr_proven =
        proven_by_map(&adr_dir).with_context(|| format!("failed to read {}", adr_dir.display()))?;

    // Reciprocity, both directions.
    for (adr, slugs) in &adr_proven {
        for slug in slugs {
            match audit_related.get(slug) {
                Some(set) if set.contains(adr) => {}
                Some(_) => errors.push(format!(
                    "::error file=docs/audit/{slug}-audit.md::{adr} lists this audit under **Proven by:** but its `related` omits {adr}"
                )),
                None => errors.push(format!(
                    "::error file=docs/adr/{}::**Proven by:** names unknown audit {slug:?}",
                    adr_filename(&adr_dir, adr).unwrap_or_else(|| format!("{adr}.md"))
                )),
            }
        }
    }
    for (slug, adrs) in &audit_related {
        for adr in adrs {
            if !adr_proven.get(adr).is_some_and(|s| s.contains(slug)) {
                errors.push(format!(
                    "::error file=docs/audit/{slug}-audit.md::`related` names {adr} but {adr} does not list this audit under **Proven by:**"
                ));
            }
        }
    }

    validate_refresh_map(root, &audit_owning, &mut errors);

    if errors.is_empty() {
        println!("Audit lane is consistent ({} audits).", audit_related.len());
        Ok(())
    } else {
        for error in &errors {
            eprintln!("{error}");
        }
        bail!("check-audit-lane found {} error(s)", errors.len());
    }
}

fn validate_frontmatter(
    errors: &mut Vec<String>,
    rel: &str,
    slug: Option<&str>,
    front: &Frontmatter,
    front_text: &str,
) {
    if front.kind.as_deref() != Some("Audit") {
        errors.push(format!(
            "::error file={rel}::frontmatter `type` must be `Audit`"
        ));
    }
    match (front.id.as_deref(), slug) {
        (Some(id), Some(slug)) if id == slug => {}
        (Some(id), Some(slug)) => errors.push(format!(
            "::error file={rel}::frontmatter `id` ({id}) must equal the filename slug ({slug})"
        )),
        (None, _) => errors.push(format!("::error file={rel}::frontmatter is missing `id`")),
        _ => {}
    }
    for (label, value) in [
        ("title", &front.title),
        ("status", &front.status),
        ("owning_surface", &front.owning_surface),
    ] {
        if !value.as_deref().is_some_and(non_empty) {
            errors.push(format!(
                "::error file={rel}::frontmatter is missing `{label}`"
            ));
        }
    }
    match front.status.as_deref() {
        Some(status) if !ALLOWED_STATUS.contains(&status) => errors.push(format!(
            "::error file={rel}::frontmatter `status` {status:?} is not one of {ALLOWED_STATUS:?}"
        )),
        _ => {}
    }
    match front.description.as_deref() {
        None => errors.push(format!(
            "::error file={rel}::frontmatter is missing `description`"
        )),
        Some(desc) => {
            let trimmed = desc.trim();
            let words = trimmed.split_whitespace().count();
            if !trimmed.ends_with('.') || trimmed.ends_with("...") || words < 6 {
                errors.push(format!(
                    "::error file={rel}::frontmatter `description` must be one hand-written sentence \
                     (>= 6 words, ending in a period, not truncated)"
                ));
            }
        }
    }
    if !front_text
        .lines()
        .any(|line| line.trim_start().starts_with("timestamp:"))
    {
        errors.push(format!(
            "::error file={rel}::frontmatter is missing `timestamp`"
        ));
    }
}

fn validate_sections(errors: &mut Vec<String>, rel: &str, slug: Option<&str>, body: &str) {
    let headings: Vec<&str> = body
        .lines()
        .filter_map(|line| line.strip_prefix("## ").map(str::trim))
        .collect();
    let expected: &[&str] = if slug == Some("dependency-gate") {
        &["Scope", "Findings", "Tracked advisories", "Evidence"]
    } else {
        &["Scope", "Findings", "Evidence"]
    };
    if headings != expected {
        errors.push(format!(
            "::error file={rel}::section skeleton must be {expected:?}, found {headings:?}"
        ));
    }
}

/// Enforces the audit↔refresh-map 1:1 correspondence and `owning_surface`
/// agreement: every `docs/audit/*-audit.md` has exactly one map entry (and vice
/// versa), each entry carries at least one refresh-trigger path, and each
/// entry's `owning_surface` matches the audit's frontmatter.
fn validate_refresh_map(
    root: &Path,
    audit_owning: &BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    const MAP: &str = ".github/config/audit-refresh-map.yml";
    let map = match load_refresh_map(root) {
        Ok(map) => map,
        Err(error) => {
            errors.push(format!("::error file={MAP}::{error}"));
            return;
        }
    };
    let mut mapped: BTreeSet<String> = BTreeSet::new();
    for entry in &map.entries {
        let Some(slug) = entry_slug(entry) else {
            errors.push(format!(
                "::error file={MAP}::entry audit path {:?} is not docs/audit/<slug>-audit.md",
                entry.audit
            ));
            continue;
        };
        if !mapped.insert(slug.to_owned()) {
            errors.push(format!("::error file={MAP}::duplicate entry for {slug}"));
        }
        if entry.refresh_triggers.paths.is_empty() {
            errors.push(format!(
                "::error file={MAP}::entry {slug} has no refresh_triggers.paths"
            ));
        }
        match audit_owning.get(slug) {
            None => errors.push(format!(
                "::error file={MAP}::entry {slug} has no docs/audit/{slug}-audit.md"
            )),
            Some(front) if front.trim() == entry.owning_surface.trim() => {}
            Some(front) => errors.push(format!(
                "::error file=docs/audit/{slug}-audit.md::`owning_surface` disagrees with the refresh map: frontmatter {front:?} vs map {:?}",
                entry.owning_surface
            )),
        }
    }
    for slug in audit_owning.keys() {
        if !mapped.contains(slug) {
            errors.push(format!(
                "::error file=docs/audit/{slug}-audit.md::no audit-refresh-map.yml entry for this audit"
            ));
        }
    }
}

fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let after = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let mut offset = 0;
    for line in after.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some((&after[..offset], &after[offset + line.len()..]));
        }
        offset += line.len();
    }
    None
}

fn collect_md(dir: &Path, exclude: &[&str]) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let is_md = path.extension().is_some_and(|ext| ext == "md");
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if is_md && !exclude.contains(&name) {
            out.push(path);
        }
    }
    Ok(out)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn is_markdown(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn is_adr_id(token: &str) -> bool {
    token
        .strip_prefix("ADR-")
        .is_some_and(|n| n.len() == 4 && n.bytes().all(|b| b.is_ascii_digit()))
}

fn adr_filename(adr_dir: &Path, adr: &str) -> Option<String> {
    let num = adr.strip_prefix("ADR-")?;
    let prefix = format!("{num}-");
    fs::read_dir(adr_dir)
        .ok()?
        .filter_map(Result::ok)
        .find_map(|entry| {
            entry
                .file_name()
                .to_str()
                .filter(|n| n.starts_with(&prefix) && is_markdown(n))
                .map(str::to_owned)
        })
}

fn adr_exists(adr_dir: &Path, token: &str) -> bool {
    adr_filename(adr_dir, token).is_some()
}

fn proven_by_map(adr_dir: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in fs::read_dir(adr_dir)? {
        let path = entry?.path();
        let name = file_name(&path);
        if !is_markdown(&name) || name == "index.md" {
            continue;
        }
        let Some(num) = name.split('-').next() else {
            continue;
        };
        if num.len() != 4 || !num.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        let slugs = proven_audit_slugs(&text);
        if !slugs.is_empty() {
            map.insert(format!("ADR-{num}"), slugs);
        }
    }
    Ok(map)
}

/// Audit slugs linked under the `**Proven by:**` block only, scanning to the
/// next `## ` heading or end of file.
fn proven_audit_slugs(text: &str) -> BTreeSet<String> {
    let mut slugs = BTreeSet::new();
    let mut in_block = false;
    for line in text.lines() {
        if line.contains("**Proven by:**") {
            in_block = true;
            continue;
        }
        if in_block {
            if line.starts_with("## ") {
                break;
            }
            if let Some(slug) = audit_slug_in_link(line) {
                slugs.insert(slug);
            }
        }
    }
    slugs
}

fn audit_slug_in_link(line: &str) -> Option<String> {
    let start = line.find("../audit/")? + "../audit/".len();
    let rest = &line[start..];
    let end = rest.find("-audit.md")?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::{audit_slug_in_link, is_adr_id, proven_audit_slugs, split_frontmatter};

    #[test]
    fn frontmatter_splits_on_fences() {
        let text = "---\ntype: Audit\nid: demo\n---\n# Demo\n\n## Scope\n";
        let (front, body) = split_frontmatter(text).expect("splits");
        assert!(front.contains("type: Audit"));
        assert!(body.starts_with("# Demo"));
        assert!(split_frontmatter("no frontmatter").is_none());
    }

    #[test]
    fn adr_ids_are_validated() {
        assert!(is_adr_id("ADR-0060"));
        assert!(!is_adr_id("ADR-60"));
        assert!(!is_adr_id("Parity-Matrix"));
        assert!(!is_adr_id("PROPERTIES-md"));
    }

    #[test]
    fn proven_by_block_extracts_audit_slugs_only_in_block() {
        let adr = "## Links\n\n- [docs/audit/error-classification-audit.md](../audit/error-classification-audit.md)\n\n**Proven by:**\n\n- [Trading Order Integrity Audit](../audit/trading-order-integrity-audit.md)\n";
        let slugs = proven_audit_slugs(adr);
        // The Links reference above the block is NOT counted; only the block is.
        assert!(slugs.contains("trading-order-integrity"));
        assert!(!slugs.contains("error-classification"));

        assert_eq!(
            audit_slug_in_link("- [X](../audit/credential-redaction-audit.md)"),
            Some("credential-redaction".to_owned())
        );
        assert_eq!(audit_slug_in_link("- plain bullet"), None);
    }
}
