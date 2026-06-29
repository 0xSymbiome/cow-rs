use std::{
    fs,
    path::{Path, PathBuf},
};

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
    pub evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRef {
    pub path: String,
    pub symbol: Option<String>,
}

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
        properties: None,
    })
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let rows = match &args.properties {
        // explicit single-file override (used by fixtures/tests)
        Some(path) => parse_property_rows(&workspace::read_to_string(path)?),
        // the registry now lives as per-family concept files under docs/properties/;
        // docs/properties/index.md keeps only the methodology + ToC.
        None => read_registry_rows(&args.repo_root)?,
    };
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

/// Collect property rows from every per-family concept file under
/// `docs/properties/`. Falls back to a single `PROPERTIES.md` for repos that
/// have not split the registry yet.
fn read_registry_rows(repo_root: &Path) -> anyhow::Result<Vec<PropertyRow>> {
    let dir = repo_root.join("docs/properties");
    let mut rows = Vec::new();
    if dir.is_dir() {
        let mut files = fs::read_dir(&dir)
            .with_context(|| format!("failed to read {}", dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "md")
                    && path.file_name().is_some_and(|name| name != "index.md")
            })
            .collect::<Vec<_>>();
        files.sort();
        for file in files {
            rows.extend(parse_property_rows(&workspace::read_to_string(&file)?));
        }
    }
    if rows.is_empty() {
        let single = repo_root.join("PROPERTIES.md");
        if single.exists() {
            rows = parse_property_rows(&workspace::read_to_string(&single)?);
        }
    }
    Ok(rows)
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
            let cells = split_table_row(trimmed);
            if cells.len() < 8 {
                return None;
            }
            Some(PropertyRow {
                id: cells[1].trim_matches('`').to_owned(),
                evidence: cells[6].to_owned(),
            })
        })
        .collect()
}

/// Splits a markdown table row on **unescaped** `|`. A cell may legitimately
/// contain an escaped pipe (`\|`) — `PROP-CON-023` uses it for the EIP-712
/// preimage notation — which `str::split('|')` would fragment, shifting every
/// later column (including `Evidence`). `|` and `\` are ASCII, so byte scanning
/// stays on `char` boundaries.
fn split_table_row(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut cells = Vec::new();
    let mut start = 0;
    for i in 0..bytes.len() {
        if bytes[i] == b'|' && (i == 0 || bytes[i - 1] != b'\\') {
            cells.push(line[start..i].trim());
            start = i + 1;
        }
    }
    cells.push(line[start..].trim());
    cells
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
    Some(EvidenceRef { path, symbol })
}

#[cfg(test)]
mod tests {
    use super::parse_property_rows;

    #[test]
    fn escaped_pipes_keep_the_evidence_column_aligned() {
        let row = r"| `PROP-CON-023` | `cow-sdk-contracts` | hash over 0x01 \| domain_separator \| struct_hash | Contract | Yes | `crates/contracts/tests/order_digest_parity_contract.rs::order_digest_fixture_rows_hold` | 2026-05-30 |";
        let rows = parse_property_rows(row);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "PROP-CON-023");
        assert_eq!(
            rows[0].evidence,
            "`crates/contracts/tests/order_digest_parity_contract.rs::order_digest_fixture_rows_hold`"
        );
    }
}
