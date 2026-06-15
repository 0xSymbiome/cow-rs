use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::policy::workspace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Override properties registry path.
    #[arg(long)]
    pub properties: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyRow {
    pub id: String,
    pub covered: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRef {
    pub path: String,
    pub symbol: Option<String>,
    pub raw: String,
}

pub fn run_default() -> anyhow::Result<()> {
    run(Args {
        repo_root: PathBuf::from("."),
        properties: None,
    })
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let path = args
        .properties
        .unwrap_or_else(|| args.repo_root.join("PROPERTIES.md"));
    let text = workspace::read_to_string(&path)?;
    let rows = parse_property_rows(&text);
    let errors = validate_rows(&args.repo_root, &rows)?;
    if errors.is_empty() {
        println!(
            "validated property citations for {} PROP row(s)",
            rows.len()
        );
        return Ok(());
    }
    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("property citations have {} error(s)", errors.len())
}

pub fn validate_rows(repo_root: &Path, rows: &[PropertyRow]) -> anyhow::Result<Vec<String>> {
    let mut errors = Vec::new();
    for row in rows {
        let refs = evidence_refs(&row.evidence);
        for reference in refs {
            validate_evidence_ref(repo_root, row, &reference, &mut errors)?;
        }
    }
    Ok(errors)
}

pub fn parse_property_rows(text: &str) -> Vec<PropertyRow> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| `PROP-") {
                return None;
            }
            let cells = trimmed.split('|').map(str::trim).collect::<Vec<_>>();
            if cells.len() < 8 {
                return None;
            }
            Some(PropertyRow {
                id: cells[1].trim_matches('`').to_owned(),
                covered: cells[5].to_owned(),
                evidence: cells[6].to_owned(),
            })
        })
        .collect()
}

pub fn evidence_refs(evidence: &str) -> Vec<EvidenceRef> {
    extract_code_spans(evidence)
        .into_iter()
        .filter_map(|span| parse_evidence_ref(&span))
        .collect()
}

fn validate_evidence_ref(
    repo_root: &Path,
    row: &PropertyRow,
    reference: &EvidenceRef,
    errors: &mut Vec<String>,
) -> anyhow::Result<()> {
    let path = repo_root.join(&reference.path);
    if !path.exists() {
        errors.push(format!(
            "{} cites missing evidence path `{}`",
            row.id, reference.path
        ));
        return Ok(());
    }
    let Some(symbol) = &reference.symbol else {
        return Ok(());
    };
    let tests = workspace::test_functions(&path)
        .with_context(|| format!("failed to inspect test symbols in {}", path.display()))?;
    if !tests.iter().any(|test| test == symbol) {
        errors.push(format!(
            "{} cites `{}` in `{}`, but it is missing or is not a test function",
            row.id, symbol, reference.path
        ));
    }
    Ok(())
}

fn extract_code_spans(text: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        spans.push(after_start[..end].to_owned());
        rest = &after_start[end + 1..];
    }
    spans
}

fn parse_evidence_ref(raw: &str) -> Option<EvidenceRef> {
    let rs_index = raw.find(".rs")?;
    let path_end = rs_index + 3;
    let path = raw[..path_end].replace('\\', "/");
    let rest = raw[path_end..].trim();
    let symbol = rest.strip_prefix("::").and_then(|tail| {
        let symbol_path = tail
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == ':')
            .collect::<String>();
        symbol_path
            .rsplit("::")
            .find(|segment| !segment.is_empty())
            .map(str::to_owned)
    });
    Some(EvidenceRef {
        path,
        symbol,
        raw: raw.to_owned(),
    })
}
