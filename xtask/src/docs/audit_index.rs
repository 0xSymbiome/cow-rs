//! Keeps `docs/audit/index.md` "Last reviewed" cells in sync with the
//! per-audit "Last reviewed" banners.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let audit_dir = args.repo_root.join("docs/audit");
    let readme_path = audit_dir.join("index.md");
    let readme = fs::read_to_string(&readme_path)
        .with_context(|| format!("required audit index missing: {}", readme_path.display()))?;

    let mut failures = Vec::new();
    let mut entries = fs::read_dir(&audit_dir)
        .with_context(|| format!("failed to read {}", audit_dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "md")
                && path.file_name().is_some_and(|name| name != "index.md")
        })
        .collect::<Vec<_>>();
    entries.sort();

    for audit in entries {
        let rel = format!(
            "docs/audit/{}",
            audit.file_name().unwrap().to_string_lossy()
        );
        let text = fs::read_to_string(&audit)
            .with_context(|| format!("failed to read {}", audit.display()))?;

        let Some(title) = text
            .lines()
            .find_map(|line| line.strip_prefix("# ").map(str::trim))
        else {
            failures.push(format!("::error file={rel}::missing top-level audit title"));
            continue;
        };
        let Some(banner_date) = text.lines().find_map(banner_review_date) else {
            failures.push(format!(
                "::error file={rel}::missing Last reviewed banner for title={title:?}"
            ));
            continue;
        };

        match index_review_date(&readme, title) {
            None => failures.push(format!(
                "::error file=docs/audit/index.md::no index row for title={title:?}"
            )),
            Some(index_date) if !is_iso_date(&index_date) => failures.push(format!(
                "::error file=docs/audit/index.md::index row for title={title:?} has invalid Last reviewed cell={index_date:?}"
            )),
            Some(index_date) if index_date != banner_date => failures.push(format!(
                "::error file=docs/audit/index.md::index_date={index_date} banner_date={banner_date} for title={title:?}"
            )),
            Some(_) => {}
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("{failure}");
        }
        bail!("audit index disagrees with {} banner(s)", failures.len());
    }
}

/// Extracts the date from an audit's review-date line: the OKF frontmatter
/// `timestamp:` key (a bare `YYYY-MM-DD` or a `YYYY-MM-DDThh:mm:ssZ` form), or
/// the legacy `last_reviewed:` / `Last reviewed:` banner.
fn banner_review_date(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed
        .strip_prefix("timestamp:")
        .or_else(|| trimmed.strip_prefix("last_reviewed:"))
        .or_else(|| trimmed.strip_prefix("Last reviewed:"))?
        .trim();
    let token = rest.split_whitespace().next()?;
    let date = token.split('T').next().unwrap_or(token);
    is_iso_date(date).then(|| date.to_owned())
}

/// Finds the "Last reviewed" cell of the index row whose first cell — plain or
/// `[Title](link)` — matches `title`. The date column is located by its header
/// label rather than a fixed position, so the index table layout can change
/// without editing this gate.
fn index_review_date(readme: &str, title: &str) -> Option<String> {
    let date_col = readme
        .lines()
        .filter(|line| line.starts_with('|'))
        .find_map(|line| {
            line.split('|')
                .position(|cell| cell.trim() == "Last reviewed")
        })?;
    for line in readme.lines().filter(|line| line.starts_with('|')) {
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() <= date_col || cells.len() < 2 {
            continue;
        }
        let first = cells[1].trim();
        let row_title = first
            .strip_prefix('[')
            .and_then(|rest| rest.split_once("]("))
            .map_or(first, |(text, _)| text);
        if row_title == title {
            return Some(cells[date_col].trim().to_owned());
        }
    }
    None
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::{banner_review_date, index_review_date, is_iso_date};

    #[test]
    fn banner_and_index_dates_parse_and_match_rows() {
        assert_eq!(
            banner_review_date("timestamp: 2026-06-10"),
            Some("2026-06-10".to_owned())
        );
        assert_eq!(
            banner_review_date("timestamp: 2026-06-10T00:00:00Z"),
            Some("2026-06-10".to_owned())
        );
        assert_eq!(
            banner_review_date("last_reviewed: 2026-06-10"),
            Some("2026-06-10".to_owned())
        );
        assert_eq!(banner_review_date("Last reviewed: soon"), None);
        assert!(is_iso_date("2026-06-10"));
        assert!(!is_iso_date("2026-6-10"));

        let readme = "| Audit | Owning surface | Status | Last reviewed |\n| --- | --- | --- | --- |\n| [Demo Audit](demo-audit.md) | x | Current | 2026-06-10 |\n";
        assert_eq!(
            index_review_date(readme, "Demo Audit"),
            Some("2026-06-10".to_owned())
        );
        assert_eq!(index_review_date(readme, "Missing"), None);
    }
}
