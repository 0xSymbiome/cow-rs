use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    let args = Args::parse(env::args().skip(1));
    if args.self_test {
        return run_self_test();
    }

    match validate_root(&args.repo_root) {
        Ok(report) => {
            println!(
                "validated {} fuzz corpus seeded propert{}",
                report.claims_checked,
                if report.claims_checked == 1 {
                    "y"
                } else {
                    "ies"
                }
            );
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::from(1)
        }
    }
}

#[derive(Debug)]
struct Args {
    repo_root: PathBuf,
    self_test: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut repo_root = PathBuf::from(".");
        let mut self_test = false;
        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--repo-root" => {
                    if let Some(value) = iter.next() {
                        repo_root = PathBuf::from(value);
                    }
                }
                "--self-test" => self_test = true,
                _ => {}
            }
        }
        Self {
            repo_root,
            self_test,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CorpusClaim {
    id: String,
    target: String,
    minimum: usize,
    requires_seed_classes: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditReport {
    claims_checked: usize,
}

fn validate_root(root: &Path) -> Result<AuditReport, Vec<String>> {
    let properties_path = root.join("PROPERTIES.md");
    let properties = match fs::read_to_string(&properties_path) {
        Ok(contents) => contents,
        Err(error) => {
            return Err(vec![format!(
                "failed to read {}: {error}",
                properties_path.display()
            )]);
        }
    };

    let claims = parse_property_claims(&properties);
    let mut errors = Vec::new();
    for claim in &claims {
        let corpus_dir = root.join("fuzz").join("corpus").join(&claim.target);
        let seeds = seed_files(&corpus_dir);
        if seeds.len() < claim.minimum {
            errors.push(format!(
                "{} `{}` claims >= {} seed file(s), found {}",
                claim.id,
                claim.target,
                claim.minimum,
                seeds.len()
            ));
        }
        if claim.requires_seed_classes {
            validate_seed_class_readme(&claim.id, &claim.target, &corpus_dir, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(AuditReport {
            claims_checked: claims.len(),
        })
    } else {
        Err(errors)
    }
}

fn parse_property_claims(properties_md: &str) -> Vec<CorpusClaim> {
    properties_md
        .lines()
        .filter_map(parse_property_row)
        .filter(|row| row.property.contains("corpus seeded"))
        .filter_map(|row| {
            let target = target_from_text(&row.property)
                .or_else(|| target_from_text(&row.evidence))
                .or_else(|| corpus_target_from_evidence(&row.evidence))?;
            let minimum = minimum_from_property(&row.property).unwrap_or_else(|| {
                if row.property.contains("non-empty") {
                    1
                } else {
                    1
                }
            });
            Some(CorpusClaim {
                id: row.id,
                target,
                minimum,
                requires_seed_classes: minimum >= 5,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PropertyRow {
    id: String,
    property: String,
    evidence: String,
}

fn parse_property_row(line: &str) -> Option<PropertyRow> {
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
        property: cells[3].to_owned(),
        evidence: cells[6].to_owned(),
    })
}

fn target_from_text(text: &str) -> Option<String> {
    extract_code_spans(text)
        .into_iter()
        .find(|span| span.starts_with("fuzz_"))
}

fn corpus_target_from_evidence(text: &str) -> Option<String> {
    extract_code_spans(text).into_iter().find_map(|span| {
        span.strip_prefix("fuzz/corpus/")
            .map(|target| target.trim_end_matches('/').to_owned())
    })
}

fn extract_code_spans(text: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        spans.push(after_start[..end].replace('\\', "/"));
        rest = &after_start[end + 1..];
    }
    spans
}

fn minimum_from_property(property: &str) -> Option<usize> {
    for marker in [">=", "at least "] {
        if let Some(value) = number_after(property, marker) {
            return Some(value);
        }
    }
    None
}

fn number_after(text: &str, marker: &str) -> Option<usize> {
    let index = text.find(marker)? + marker.len();
    let digits = text[index..]
        .chars()
        .skip_while(|ch| ch.is_ascii_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn seed_files(corpus_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(corpus_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_none_or(|name| !name.eq_ignore_ascii_case("README.md"))
        })
        .collect()
}

fn validate_seed_class_readme(
    id: &str,
    target: &str,
    corpus_dir: &Path,
    errors: &mut Vec<String>,
) {
    let readme_path = corpus_dir.join("README.md");
    let Ok(readme) = fs::read_to_string(&readme_path) else {
        errors.push(format!(
            "{id} `{target}` claims a >= 5 corpus but has no corpus README.md"
        ));
        return;
    };
    for class in ["canonical", "boundary", "adversarial"] {
        if !readme.to_ascii_lowercase().contains(class) {
            errors.push(format!(
                "{id} `{target}` corpus README.md does not document the {class} seed class"
            ));
        }
    }
}

fn run_self_test() -> ExitCode {
    let root = env::temp_dir().join(format!(
        "cow-rs-fuzz-corpus-audit-self-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let corpus = root.join("fuzz").join("corpus").join("fuzz_self_test");
    if let Err(error) = fs::create_dir_all(&corpus)
        .and_then(|()| fs::write(corpus.join("seed.bin"), b"seed"))
        .and_then(|()| fs::write(corpus.join("README.md"), SELF_TEST_README))
        .and_then(|()| fs::write(root.join("PROPERTIES.md"), SELF_TEST_PROPERTIES))
    {
        eprintln!("self-test setup failed: {error}");
        let _ = fs::remove_dir_all(&root);
        return ExitCode::from(1);
    }

    let result = validate_root(&root);
    let _ = fs::remove_dir_all(&root);
    match result {
        Ok(_) => {
            eprintln!("self-test failed: tampered corpus claim was accepted");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::from(1)
        }
    }
}

const SELF_TEST_PROPERTIES: &str = r#"
| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-TEST-001` | `cow-sdk-core` | `fuzz_self_test` ships with a corpus seeded from fixture data with at least 2 seed files. | Contract | Yes | `fuzz/corpus/fuzz_self_test/` | 2026-05-01 |
"#;

const SELF_TEST_README: &str = r#"
# `fuzz_self_test` Corpus

- canonical: fixture seed.
- boundary: boundary seed.
- adversarial: adversarial seed.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_reports_when_property_claims_more_seeds_than_directory_has() {
        let root = unique_temp_root();
        let corpus = root.join("fuzz").join("corpus").join("fuzz_self_test");
        fs::create_dir_all(&corpus).expect("self-test corpus dir must be created");
        fs::write(corpus.join("seed.bin"), b"seed").expect("self-test seed must be written");
        fs::write(corpus.join("README.md"), SELF_TEST_README)
            .expect("self-test README must be written");
        fs::write(root.join("PROPERTIES.md"), SELF_TEST_PROPERTIES)
            .expect("self-test properties must be written");

        let errors = validate_root(&root).expect_err("tampered fixture must fail validation");
        assert!(
            errors.iter().any(|error| error.contains("claims >= 2")),
            "tampered fixture must report the inflated seed claim: {errors:?}",
        );
        fs::remove_dir_all(root).expect("self-test temp root must be removed");
    }

    #[test]
    fn parse_property_claims_reads_target_and_minimum_from_property_rows() {
        let claims = parse_property_claims(SELF_TEST_PROPERTIES);
        assert_eq!(
            claims,
            vec![CorpusClaim {
                id: "PROP-TEST-001".to_owned(),
                target: "fuzz_self_test".to_owned(),
                minimum: 2,
                requires_seed_classes: false,
            }]
        );
    }

    fn unique_temp_root() -> PathBuf {
        let root = env::temp_dir().join(format!(
            "cow-rs-fuzz-corpus-audit-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("self-test temp root must be created");
        root
    }
}
