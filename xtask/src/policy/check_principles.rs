//! Principle-document gate behind `cargo check-principles`.
//!
//! `check-adr-coverage` proves the principle/ADR *map* is internally consistent
//! and that every accepted ADR is covered. This gate proves the principle
//! *documents* stay consistent with that map, so the principle→ADR edge cannot
//! drift across its three copies (the map, each file's `anchored_by`
//! frontmatter, and the rendered `**Anchored by**` line). It also enforces the
//! shape-aware skeleton and keeps inbound deep-links pointed at the per-file
//! slugs rather than stale index anchors.
//!
//! What it checks:
//!   * every map principle has a unique non-empty `slug`, and `slug` ↔ file is
//!     a 1:1 correspondence under `docs/principles/` (no orphan files, no
//!     missing files);
//!   * each principle file's frontmatter is well-formed (`type: Principle`,
//!     `title`, `description`, `tags`, `timestamp`, `anchored_by`, `shape`);
//!   * `shape` is one of `rule`/`structure`/`classify`/`pipeline`;
//!   * the frontmatter `anchored_by` set and the prose `**Anchored by**` line
//!     both equal the map's primary + supporting ADRs, primary first;
//!   * the required skeleton sections are present, plus a `mermaid` diagram for
//!     every non-`rule` shape;
//!   * relative `../adr/NNNN-*.md` links resolve to real files;
//!   * `docs/principles/index.md` links every principle exactly once;
//!   * inbound principle links across README + `docs/**` resolve to a real
//!     `<slug>.md` and never use a stale `index.md#<slug>` anchor.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::policy::{fixtures, workspace};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

#[derive(Debug, Deserialize)]
struct PrincipleAdrMap {
    principles: Vec<MapEntry>,
}

#[derive(Debug, Deserialize)]
struct MapEntry {
    id: u32,
    name: String,
    slug: String,
    primary_adr: String,
    #[serde(default)]
    supporting_adrs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    #[serde(rename = "type")]
    kind: Option<String>,
    title: Option<String>,
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    timestamp: Option<String>,
    #[serde(default)]
    anchored_by: Vec<String>,
    shape: Option<String>,
}

const SHAPES: [&str; 4] = ["rule", "structure", "classify", "pipeline"];
const REQUIRED_SECTIONS: [&str; 4] = [
    "**Invariant**",
    "**Why**",
    "**Enforced by**",
    "**Anchored by**",
];

pub fn run(args: &Args) -> anyhow::Result<()> {
    let principles_dir = args.repo_root.join("docs/principles");
    let map_path = args.repo_root.join(".github/config/principle-adr-map.yaml");
    let map: PrincipleAdrMap = fixtures::load_yaml(&map_path)
        .with_context(|| format!("failed to load {}", map_path.display()))?;

    let mut errors = Vec::new();

    // Map side: unique, non-empty slugs.
    let mut by_slug: BTreeMap<String, &MapEntry> = BTreeMap::new();
    for entry in &map.principles {
        let slug = entry.slug.trim();
        if slug.is_empty() {
            errors.push(format!(
                "principle {} `{}` has an empty slug in the map",
                entry.id, entry.name
            ));
            continue;
        }
        if by_slug.insert(slug.to_owned(), entry).is_some() {
            errors.push(format!("duplicate principle slug `{slug}` in the map"));
        }
    }
    let slugs: BTreeSet<String> = by_slug.keys().cloned().collect();

    // File side: every `<slug>.md` exists, no orphans, each validates.
    let mut seen = BTreeSet::new();
    let dir = fs::read_dir(&principles_dir)
        .with_context(|| format!("failed to read {}", principles_dir.display()))?;
    for entry in dir {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_owned();
        if stem == "index" {
            continue;
        }
        seen.insert(stem.clone());
        match by_slug.get(&stem) {
            None => errors.push(format!(
                "docs/principles/{stem}.md has no entry in principle-adr-map.yaml"
            )),
            Some(entry) => validate_file(&mut errors, &args.repo_root, &path, entry),
        }
    }
    for (slug, entry) in &by_slug {
        if !seen.contains(slug) {
            errors.push(format!(
                "principle {} `{}` maps to missing file docs/principles/{}.md",
                entry.id, entry.name, slug
            ));
        }
    }

    validate_index(&mut errors, &principles_dir, &slugs);
    validate_inbound_links(&mut errors, &args.repo_root, &slugs);

    if errors.is_empty() {
        println!(
            "principles gate: {} principle(s) consistent with the ADR map",
            map.principles.len()
        );
        return Ok(());
    }
    errors.sort();
    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("principles gate has {} error(s)", errors.len());
}

fn validate_file(errors: &mut Vec<String>, repo_root: &Path, path: &Path, entry: &MapEntry) {
    let slug = &entry.slug;
    let text = match workspace::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            errors.push(format!("failed to read docs/principles/{slug}.md: {error}"));
            return;
        }
    };
    let Some((front_text, body)) = split_frontmatter(&text) else {
        errors.push(format!(
            "docs/principles/{slug}.md is missing a `---` frontmatter block"
        ));
        return;
    };
    let front: Frontmatter = match serde_norway::from_str(&front_text) {
        Ok(front) => front,
        Err(error) => {
            errors.push(format!(
                "docs/principles/{slug}.md frontmatter does not parse: {error}"
            ));
            return;
        }
    };

    validate_frontmatter(errors, slug, &front);
    validate_linkage(errors, slug, &front, &body, entry);
    validate_sections(
        errors,
        slug,
        &body,
        front.shape.as_deref().unwrap_or_default(),
    );
    validate_adr_links(errors, slug, repo_root, &body);
}

/// Frontmatter carries the required keys and a known `shape`.
fn validate_frontmatter(errors: &mut Vec<String>, slug: &str, front: &Frontmatter) {
    if front.kind.as_deref() != Some("Principle") {
        errors.push(format!(
            "docs/principles/{slug}.md frontmatter `type` must be `Principle`"
        ));
    }
    for (label, present) in [
        ("title", front.title.as_deref().is_some_and(non_empty)),
        (
            "description",
            front.description.as_deref().is_some_and(non_empty),
        ),
        ("tags", !front.tags.is_empty()),
        (
            "timestamp",
            front.timestamp.as_deref().is_some_and(non_empty),
        ),
        ("anchored_by", !front.anchored_by.is_empty()),
    ] {
        if !present {
            errors.push(format!(
                "docs/principles/{slug}.md frontmatter is missing `{label}`"
            ));
        }
    }
    let shape = front.shape.as_deref().unwrap_or_default();
    if !SHAPES.contains(&shape) {
        errors.push(format!(
            "docs/principles/{slug}.md frontmatter `shape` must be one of {SHAPES:?}, got `{shape}`"
        ));
    }
}

/// The frontmatter `anchored_by` and the prose `**Anchored by**` line both equal
/// the map's ADR set, primary first.
fn validate_linkage(
    errors: &mut Vec<String>,
    slug: &str,
    front: &Frontmatter,
    body: &str,
    entry: &MapEntry,
) {
    let expected = expected_ids(entry);
    let primary = normalize_adr(&entry.primary_adr);

    let front_ids = collect_adr_ids(&front.anchored_by.join(" "));
    if to_set(&front_ids) != expected {
        errors.push(format!(
            "docs/principles/{slug}.md frontmatter `anchored_by` {:?} does not match the map {:?}",
            to_set(&front_ids),
            expected
        ));
    }
    if front_ids.first() != primary.as_ref() {
        errors.push(format!(
            "docs/principles/{slug}.md frontmatter `anchored_by` must list the primary ADR ({}) first",
            entry.primary_adr
        ));
    }

    let Some(line) = anchored_by_line(body) else {
        errors.push(format!(
            "docs/principles/{slug}.md has no `**Anchored by**:` line"
        ));
        return;
    };
    let prose_ids = collect_adr_ids(line);
    if to_set(&prose_ids) != expected {
        errors.push(format!(
            "docs/principles/{slug}.md `**Anchored by**` line {:?} does not match the map {:?}",
            to_set(&prose_ids),
            expected
        ));
    }
    if prose_ids.first() != primary.as_ref() {
        errors.push(format!(
            "docs/principles/{slug}.md `**Anchored by**` line must name the primary ADR ({}) first",
            entry.primary_adr
        ));
    }
}

/// Required skeleton sections present, plus a mermaid diagram for non-`rule` shapes.
fn validate_sections(errors: &mut Vec<String>, slug: &str, body: &str, shape: &str) {
    for section in REQUIRED_SECTIONS {
        if !body.contains(section) {
            errors.push(format!(
                "docs/principles/{slug}.md is missing the `{section}` section"
            ));
        }
    }
    let diagram = match shape {
        "structure" => Some("**Shape**"),
        "classify" => Some("**Decision**"),
        "pipeline" => Some("**Pipeline**"),
        _ => None,
    };
    if let Some(section) = diagram {
        let has_diagram = body.contains(section) && body.contains("```mermaid");
        if !has_diagram {
            errors.push(format!(
                "docs/principles/{slug}.md has shape `{shape}` and must carry a `{section}` section with a ```mermaid diagram"
            ));
        }
    }
}

/// Relative `../adr/NNNN-*.md` links resolve to real files.
fn validate_adr_links(errors: &mut Vec<String>, slug: &str, repo_root: &Path, body: &str) {
    for target in adr_link_targets(body) {
        let resolved = repo_root.join("docs/adr").join(&target);
        if !resolved.exists() {
            errors.push(format!(
                "docs/principles/{slug}.md links to ../adr/{target}, which does not exist"
            ));
        }
    }
}

fn validate_index(errors: &mut Vec<String>, principles_dir: &Path, slugs: &BTreeSet<String>) {
    let index_path = principles_dir.join("index.md");
    let text = match workspace::read_to_string(&index_path) {
        Ok(text) => text,
        Err(error) => {
            errors.push(format!("failed to read docs/principles/index.md: {error}"));
            return;
        }
    };
    for slug in slugs {
        if !text.contains(&format!("({slug}.md)")) {
            errors.push(format!(
                "docs/principles/index.md does not link to {slug}.md"
            ));
        }
    }
    for linked in markdown_link_stems(&text) {
        if linked != "index" && !slugs.contains(&linked) {
            errors.push(format!(
                "docs/principles/index.md links to {linked}.md, which is not a known principle"
            ));
        }
    }
}

fn validate_inbound_links(errors: &mut Vec<String>, repo_root: &Path, slugs: &BTreeSet<String>) {
    let mut files = Vec::new();
    let readme = repo_root.join("README.md");
    if readme.exists() {
        files.push(readme);
    }
    collect_markdown(&repo_root.join("docs"), &mut files);

    for file in files {
        // The index owns same-directory `<slug>.md` links; skip it here.
        if file.ends_with("docs/principles/index.md") {
            continue;
        }
        let Ok(text) = workspace::read_to_string(&file) else {
            continue;
        };
        let rel = workspace::relative_path(repo_root, &file);
        for reference in principle_references(&text) {
            match reference {
                PrincipleRef::StaleAnchor(slug) if slugs.contains(&slug) => errors.push(format!(
                    "{rel} links to principles/index.md#{slug}; deep-link principles/{slug}.md instead"
                )),
                PrincipleRef::File(slug) if !slugs.contains(&slug) => errors.push(format!(
                    "{rel} links to principles/{slug}.md, which is not a known principle"
                )),
                _ => {}
            }
        }
    }
}

// --- parsing helpers -------------------------------------------------------

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

/// Splits a `---`-delimited frontmatter block from the body. Returns `None`
/// when the text does not open with a frontmatter fence or never closes it.
fn split_frontmatter(text: &str) -> Option<(String, String)> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return None;
    }
    let mut front = String::new();
    let mut body = String::new();
    let mut in_front = true;
    for line in lines {
        if in_front {
            if line.trim() == "---" {
                in_front = false;
                continue;
            }
            front.push_str(line);
            front.push('\n');
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    if in_front {
        return None;
    }
    Some((front, body))
}

/// The expected 4-digit ADR id set for a map entry (primary + supporting).
fn expected_ids(entry: &MapEntry) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    if let Some(primary) = normalize_adr(&entry.primary_adr) {
        ids.insert(primary);
    }
    for adr in &entry.supporting_adrs {
        if let Some(id) = normalize_adr(adr) {
            ids.insert(id);
        }
    }
    ids
}

fn to_set(ids: &[String]) -> BTreeSet<String> {
    ids.iter().cloned().collect()
}

/// Normalizes a single ADR token (`0012`, `ADR-0012`, `ADR 0012`) to its
/// 4-digit id.
fn normalize_adr(token: &str) -> Option<String> {
    collect_adr_ids(token).into_iter().next().or_else(|| {
        let digits: String = token.chars().filter(char::is_ascii_digit).collect();
        (digits.len() == 4).then_some(digits)
    })
}

/// Extracts every `ADR[ -]NNNN` 4-digit id from free text, in order. Anchored
/// on the literal `ADR` so unrelated 4-digit runs (e.g. `EIP-1271`) are ignored.
fn collect_adr_ids(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut ids = Vec::new();
    let mut i = 0;
    while i + 3 <= chars.len() {
        if chars[i] == 'A' && chars[i + 1] == 'D' && chars[i + 2] == 'R' {
            let mut k = i + 3;
            while k < chars.len() && (chars[k] == ' ' || chars[k] == '-') {
                k += 1;
            }
            let mut digits = String::new();
            while k < chars.len() && chars[k].is_ascii_digit() && digits.len() < 4 {
                digits.push(chars[k]);
                k += 1;
            }
            if digits.len() == 4 {
                ids.push(digits);
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    ids
}

fn anchored_by_line(body: &str) -> Option<&str> {
    body.lines().find(|line| line.contains("**Anchored by**"))
}

/// Relative ADR link targets (the `NNNN-*.md` filename, anchor stripped) found
/// in `(../adr/...)` link destinations.
fn adr_link_targets(body: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let needle = "(../adr/";
    let mut rest = body;
    while let Some(pos) = rest.find(needle) {
        rest = &rest[pos + needle.len()..];
        let end = rest
            .find(|ch: char| ch == ')' || ch == '#' || ch.is_whitespace())
            .unwrap_or(rest.len());
        let target = &rest[..end];
        if !target.is_empty() {
            targets.push(target.to_owned());
        }
    }
    targets
}

/// Stems of same-directory `(<stem>.md)` markdown links.
fn markdown_link_stems(text: &str) -> Vec<String> {
    let mut stems = Vec::new();
    let needle = "](";
    let mut rest = text;
    while let Some(pos) = rest.find(needle) {
        rest = &rest[pos + needle.len()..];
        let end = rest.find(')').unwrap_or(rest.len());
        let target = &rest[..end];
        match target.strip_suffix(".md") {
            Some(stem) if !stem.contains('/') => stems.push(stem.to_owned()),
            _ => {}
        }
    }
    stems
}

enum PrincipleRef {
    File(String),
    StaleAnchor(String),
}

/// Every `principles/...` link destination in a document, classified as a
/// per-file deep-link or a stale `index.md#<anchor>` reference.
fn principle_references(text: &str) -> Vec<PrincipleRef> {
    let mut refs = Vec::new();
    let needle = "principles/";
    let mut rest = text;
    while let Some(pos) = rest.find(needle) {
        rest = &rest[pos + needle.len()..];
        let end = rest.find([')', '"', ' ', ']', '\n']).unwrap_or(rest.len());
        let tail = &rest[..end];
        if let Some(anchor) = tail.strip_prefix("index.md#") {
            refs.push(PrincipleRef::StaleAnchor(anchor.to_owned()));
        } else if let Some(stem) = tail.strip_suffix(".md").filter(|stem| *stem != "index") {
            refs.push(PrincipleRef::File(stem.to_owned()));
        }
    }
    refs
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}
