//! Stale-phrase lint walker.
//!
//! Walks the supplied scan root for markdown and Rust source files. For each
//! file, mask out `<!-- audit-trail-stale: ID -->` ...
//! `<!-- /audit-trail-stale -->` quarantine blocks so verbatim historical
//! quotes can stay in the source for traceability. Then run every regex in
//! [`crate::stale_phrase_catalog::STALE_PHRASE_PATTERNS`] over the masked
//! text and report the first match per (file, pattern) pair.
//!
//! Acceptance criteria:
//!
//! - Unbalanced quarantine markers (an opening without a matching close, a
//!   close without an opening, or nested quarantines) are themselves lint
//!   failures.
//! - The legacy `<!-- gpt-stale -->` quarantine marker is rejected. Only
//!   `<!-- audit-trail-stale: ID -->` is recognized.
//! - Patterns inside a recognized quarantine block are tolerated.
//! - Patterns outside any quarantine block fail the lint.
//!
//! This lint is invoked via the `strategy-doc-lint` cargo alias as a local
//! pre-push procedure. It is not wired as a required-for-merge CI gate.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::collect_relative_files;
use crate::stale_phrase_catalog::{STALE_PHRASE_PATTERNS, StalePattern};

const OPEN_MARKER_PREFIX: &str = "<!-- audit-trail-stale:";
const OPEN_MARKER_SUFFIX: &str = "-->";
const CLOSE_MARKER: &str = "<!-- /audit-trail-stale -->";
const REJECTED_GPT_OPEN: &str = "<!-- gpt-stale";
const REJECTED_GPT_CLOSE: &str = "<!-- /gpt-stale -->";

/// Run the lint over every markdown and Rust file under `root` and fail with
/// the first regression found.
pub(crate) fn run(root: &Path) -> Result<()> {
    let mut checked = 0_usize;
    if !root.exists() {
        println!("validated 0 documents (root does not exist)");
        return Ok(());
    }

    let compiled: Vec<(StalePattern, Regex)> = STALE_PHRASE_PATTERNS
        .iter()
        .map(|entry| {
            let regex = Regex::new(entry.pattern)
                .with_context(|| format!("pattern {} failed to compile", entry.id))?;
            Ok((*entry, regex))
        })
        .collect::<Result<_>>()?;

    for (relative, path) in collect_relative_files(root)? {
        if !is_target_file(&relative) {
            continue;
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        check_text(&text, &path, &compiled)?;
        checked += 1;
    }
    println!(
        "validated {checked} document(s) against {} pattern(s)",
        compiled.len()
    );
    Ok(())
}

fn is_target_file(relative: &str) -> bool {
    relative.ends_with(".md") || relative.ends_with(".rs")
}

fn check_text(text: &str, path: &Path, compiled: &[(StalePattern, Regex)]) -> Result<()> {
    if text.contains(REJECTED_GPT_OPEN) || text.contains(REJECTED_GPT_CLOSE) {
        bail!(
            "{} uses the legacy `gpt-stale` quarantine marker; migrate to `audit-trail-stale`",
            path.display()
        );
    }
    let masked = mask_quarantine_blocks(text, path)?;
    for (entry, regex) in compiled {
        if let Some(found) = regex.find(&masked) {
            bail!(
                "{} matches stale-phrase pattern `{}` at byte offset {}: {}",
                path.display(),
                entry.id,
                found.start(),
                truncate(found.as_str())
            );
        }
    }
    Ok(())
}

fn truncate(text: &str) -> String {
    const LIMIT: usize = 80;
    if text.len() <= LIMIT {
        text.to_string()
    } else {
        format!("{}...", &text[..LIMIT])
    }
}

/// Replace the contents of every `<!-- audit-trail-stale: ID -->` block with
/// equivalent-length whitespace so the inner text is invisible to the
/// pattern scan. Unbalanced or nested quarantine markers are themselves
/// lint failures.
fn mask_quarantine_blocks(text: &str, path: &Path) -> Result<String> {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;
    loop {
        match remaining.find(OPEN_MARKER_PREFIX) {
            None => {
                if remaining.contains(CLOSE_MARKER) {
                    bail!(
                        "{} contains a `<!-- /audit-trail-stale -->` close marker without a matching open",
                        path.display()
                    );
                }
                output.push_str(remaining);
                return Ok(output);
            }
            Some(open_idx) => {
                output.push_str(&remaining[..open_idx]);
                let after_prefix = &remaining[open_idx..];
                let header_end = after_prefix.find(OPEN_MARKER_SUFFIX).with_context(|| {
                    format!(
                        "{}: open quarantine marker is missing its `-->` close",
                        path.display()
                    )
                })?;
                let header_end_abs = header_end + OPEN_MARKER_SUFFIX.len();
                let body_and_close = &after_prefix[header_end_abs..];
                let close_idx = body_and_close.find(CLOSE_MARKER).with_context(|| {
                    format!(
                        "{}: open `audit-trail-stale` quarantine has no matching close marker",
                        path.display()
                    )
                })?;
                let body = &body_and_close[..close_idx];
                if body.contains(OPEN_MARKER_PREFIX) {
                    bail!(
                        "{} contains a nested `audit-trail-stale` quarantine block (not allowed)",
                        path.display()
                    );
                }
                // Replace header + body + close with whitespace of the same
                // length so byte offsets reported later still line up with
                // the original source text.
                let consumed_len = header_end_abs + body.len() + CLOSE_MARKER.len();
                for ch in remaining[open_idx..open_idx + consumed_len].chars() {
                    if ch == '\n' {
                        output.push('\n');
                    } else {
                        output.push(' ');
                    }
                }
                remaining = &remaining[open_idx + consumed_len..];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn compile_catalog() -> Vec<(StalePattern, Regex)> {
        STALE_PHRASE_PATTERNS
            .iter()
            .map(|entry| (*entry, Regex::new(entry.pattern).unwrap()))
            .collect()
    }

    #[test]
    fn clean_text_passes() {
        let compiled = compile_catalog();
        let path = PathBuf::from("clean.md");
        let text = "# Heading\n\nA normal English paragraph with no banned phrases.\n";
        assert!(check_text(text, &path, &compiled).is_ok());
    }

    #[test]
    fn matching_text_outside_quarantine_fails() {
        let compiled = compile_catalog();
        let path = PathBuf::from("regression.md");
        let text = "Workspace already supports Lens, the strategy concluded.";
        assert!(check_text(text, &path, &compiled).is_err());
    }

    #[test]
    fn matching_text_inside_quarantine_is_tolerated() {
        let compiled = compile_catalog();
        let path = PathBuf::from("audit-quote.md");
        let text = concat!(
            "Header line.\n",
            "<!-- audit-trail-stale: F4-001 -->\n",
            "> Workspace already supports Lens (verbatim quote for traceability).\n",
            "<!-- /audit-trail-stale -->\n",
            "Body line.\n"
        );
        assert!(check_text(text, &path, &compiled).is_ok());
    }

    #[test]
    fn nested_quarantine_fails() {
        let compiled = compile_catalog();
        let path = PathBuf::from("nested.md");
        let text = concat!(
            "<!-- audit-trail-stale: A -->\n",
            "<!-- audit-trail-stale: B -->\n",
            "inner\n",
            "<!-- /audit-trail-stale -->\n",
            "<!-- /audit-trail-stale -->\n",
        );
        assert!(check_text(text, &path, &compiled).is_err());
    }

    #[test]
    fn unbalanced_quarantine_fails() {
        let compiled = compile_catalog();
        let path = PathBuf::from("unbalanced.md");
        let text = "<!-- audit-trail-stale: A -->\nbody\n";
        assert!(check_text(text, &path, &compiled).is_err());
    }

    #[test]
    fn close_without_open_fails() {
        let compiled = compile_catalog();
        let path = PathBuf::from("close-only.md");
        let text = "body\n<!-- /audit-trail-stale -->\n";
        assert!(check_text(text, &path, &compiled).is_err());
    }

    #[test]
    fn legacy_gpt_marker_rejected() {
        let compiled = compile_catalog();
        let path = PathBuf::from("legacy.md");
        let text = "<!-- gpt-stale -->\nbody\n<!-- /gpt-stale -->\n";
        assert!(check_text(text, &path, &compiled).is_err());
    }
}
