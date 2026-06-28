//! The repository's release-version surface.
//!
//! Every human-facing place that pins the SDK's published version: README
//! install snippets, the crates.io badge, the "is published" lines, the npm
//! install command, and the npm package template.
//!
//! Cargo manifests are bumped by cargo-release itself; this module owns the
//! *documentation* pins it cannot reach. [`rewrite`] runs from the release hook
//! and rewrites them all to the release version; [`scan`] backs the
//! `check-workspace-versions` gate so a drifted pin fails CI rather than
//! shipping stale install instructions.
//!
//! Matching is anchored: each pattern keys off a fixed identifier
//! (`cow-sdk…`, `@symbiome-forge/cow-sdk-wasm@`, `message=v`) so it touches only
//! install-pin contexts. Narrative or historical references — the changelog
//! version headings, "the first release (`0.1.0-alpha.1`) has shipped" — carry
//! no such anchor and are deliberately left alone.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Context, Result};
use regex::{Captures, Regex};

/// An anchored install-pin pattern. `regex` captures `pre` (the fixed anchor),
/// `ver` (the version token to rewrite), and optionally `post` (a trailing
/// delimiter such as a closing quote or backtick).
struct Pattern {
    regex: Regex,
}

impl Pattern {
    fn new(source: &str) -> Self {
        Self {
            regex: Regex::new(source).expect("version-surface pattern is a valid regex"),
        }
    }
}

static PATTERNS: LazyLock<Vec<Pattern>> = LazyLock::new(|| {
    [
        // `cow-sdk = "X"` and `cow-sdk-core = "X"` (Cargo install snippets).
        r#"(?P<pre>cow-sdk(?:-[a-z]+)* = ")(?P<ver>[0-9][^"]*)(?P<post>")"#,
        // `cow-sdk = { version = "X", … }` (feature-bearing snippets).
        r#"(?P<pre>cow-sdk(?:-[a-z]+)* = \{ version = ")(?P<ver>[0-9][^"]*)(?P<post>")"#,
        // `cargo add cow-sdk@X`.
        r"(?P<pre>cow-sdk(?:-[a-z]+)*@)(?P<ver>[0-9][0-9A-Za-z.+-]*)",
        // `npm install @symbiome-forge/cow-sdk-wasm@X`.
        r"(?P<pre>@symbiome-forge/cow-sdk-wasm@)(?P<ver>[0-9][0-9A-Za-z.+-]*)",
        // The static crates.io badge: `…&message=vX&…`.
        r"(?P<pre>message=v)(?P<ver>[0-9][0-9A-Za-z.+-]*)",
        // Backtick "is published" prose: `` `cow-sdk` `X` ``.
        r"(?P<pre>`cow-sdk` `)(?P<ver>[0-9][^`]*)(?P<post>`)",
        // Backtick "is published" prose for the npm package.
        r"(?P<pre>`@symbiome-forge/cow-sdk-wasm` `)(?P<ver>[0-9][^`]*)(?P<post>`)",
    ]
    .into_iter()
    .map(Pattern::new)
    .collect()
});

/// A single discovered install-pin, for the drift gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pin {
    pub file: PathBuf,
    pub line: usize,
    pub version: String,
}

/// Rewrites every install-pin in the documentation surface to `version`.
///
/// Returns the files actually changed. Idempotent: re-running with the same
/// version is a no-op, so cargo-release's once-per-crate hook invocation is
/// safe.
pub fn rewrite(root: &Path, version: &str) -> Result<Vec<PathBuf>> {
    let mut changed = Vec::new();
    for file in surface_files(root)? {
        let original = fs::read_to_string(&file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        let mut updated = original.clone();
        for pattern in PATTERNS.iter() {
            updated = pattern
                .regex
                .replace_all(&updated, |caps: &Captures<'_>| substitute(caps, version))
                .into_owned();
        }
        if updated != original {
            fs::write(&file, &updated)
                .with_context(|| format!("failed to write {}", file.display()))?;
            changed.push(file);
        }
    }
    if rewrite_npm_template(root, version)? {
        changed.push(root.join(NPM_TEMPLATE));
    }
    Ok(changed)
}

/// Scans the documentation surface and returns every install-pin found.
///
/// The `check-workspace-versions` gate compares each against the workspace
/// version.
pub fn scan(root: &Path) -> Result<Vec<Pin>> {
    let mut pins = Vec::new();
    for file in surface_files(root)? {
        let text = fs::read_to_string(&file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        for pattern in PATTERNS.iter() {
            for caps in pattern.regex.captures_iter(&text) {
                let matched = caps.name("ver").expect("every pattern captures `ver`");
                pins.push(Pin {
                    file: file.clone(),
                    line: text[..matched.start()]
                        .bytes()
                        .filter(|&b| b == b'\n')
                        .count()
                        + 1,
                    version: matched.as_str().to_owned(),
                });
            }
        }
    }
    Ok(pins)
}

fn substitute(caps: &Captures<'_>, version: &str) -> String {
    let pre = &caps["pre"];
    let post = caps.name("post").map_or("", |m| m.as_str());
    format!("{pre}{version}{post}")
}

const NPM_TEMPLATE: &str = "crates/js/npm/package.template.json";

/// Sets the top-level `"version"` field of the npm package template.
///
/// Returns whether the file changed. (The template's value is also gated by
/// `check-workspace-versions`; this keeps it in lockstep on release.)
fn rewrite_npm_template(root: &Path, version: &str) -> Result<bool> {
    let path = root.join(NPM_TEMPLATE);
    if !path.is_file() {
        return Ok(false);
    }
    let original =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    // The top-level "version" is the first such key in the template.
    let regex = Regex::new(r#"(?P<pre>"version"\s*:\s*")[^"]*(?P<post>")"#)
        .expect("npm template version pattern is a valid regex");
    let updated = regex.replace(&original, |caps: &Captures<'_>| substitute(caps, version));
    if updated == original {
        return Ok(false);
    }
    fs::write(&path, updated.as_ref())
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

/// The documentation files that carry install pins.
///
/// The root README, every crate README, the npm package README, and the
/// top-level docs guides.
fn surface_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = vec![root.join("README.md")];

    let crates = root.join("crates");
    for entry in
        fs::read_dir(&crates).with_context(|| format!("failed to read {}", crates.display()))?
    {
        let readme = entry?.path().join("README.md");
        if readme.is_file() {
            files.push(readme);
        }
    }
    files.push(root.join("crates/js/npm/README.md"));

    let docs = root.join("docs");
    if docs.is_dir() {
        for entry in
            fs::read_dir(&docs).with_context(|| format!("failed to read {}", docs.display()))?
        {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }

    files.retain(|file| file.is_file());
    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite_text(input: &str, version: &str) -> String {
        let mut out = input.to_owned();
        for pattern in PATTERNS.iter() {
            out = pattern
                .regex
                .replace_all(&out, |caps: &Captures<'_>| substitute(caps, version))
                .into_owned();
        }
        out
    }

    #[test]
    fn rewrites_every_install_pin_shape() {
        let input = r#"
cow-sdk = "0.1.0-alpha.1"
cow-sdk-core = "0.1.0-alpha.1"
cow-sdk-trading = { version = "0.1.0-alpha.1", features = ["tracing"] }
cargo add cow-sdk@0.1.0-alpha.1
npm install @symbiome-forge/cow-sdk-wasm@0.1.0-alpha.1
[![crates.io](https://img.shields.io/static/v1?label=crates.io&message=v0.1.0-alpha.1&color=e6a96d)]
`cow-sdk` `0.1.0-alpha.1` is published and `@symbiome-forge/cow-sdk-wasm` `0.1.0-alpha.1` on npm.
"#;
        let out = rewrite_text(input, "0.1.0-alpha.3");
        assert!(
            !out.contains("0.1.0-alpha.1"),
            "a pin was left stale:\n{out}"
        );
        // Eight pins: five Cargo/cargo-add/npm/badge plus the two backtick
        // "is published" versions on the final line.
        assert_eq!(out.matches("0.1.0-alpha.3").count(), 8);
    }

    #[test]
    fn leaves_narrative_and_historical_references_alone() {
        // No `cow-sdk`/`message=v`/npm anchor, so these must not be rewritten.
        let input = "\
## [0.1.0-alpha.1] - 2026-06-15
The first release (`0.1.0-alpha.1`) has shipped.
Measured on the `0.1.0-alpha.1` build.
The alpha line is `0.1.0-alpha`.";
        assert_eq!(rewrite_text(input, "0.1.0-alpha.3"), input);
    }

    #[test]
    fn scan_reports_pins_with_line_numbers() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("crates/core")).unwrap();
        fs::write(
            root.join("README.md"),
            "intro\ncow-sdk = \"0.1.0-alpha.1\"\n",
        )
        .unwrap();
        fs::write(
            root.join("crates/core/README.md"),
            "cow-sdk-core = \"0.1.0-alpha.3\"\n",
        )
        .unwrap();

        let pins = scan(root).unwrap();
        let root_pin = pins
            .iter()
            .find(|p| p.file.ends_with("README.md") && p.file.parent() == Some(root))
            .unwrap();
        assert_eq!(root_pin.version, "0.1.0-alpha.1");
        assert_eq!(root_pin.line, 2);
    }

    #[test]
    fn rewrite_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("crates")).unwrap();
        fs::write(root.join("README.md"), "cow-sdk = \"0.1.0-alpha.1\"\n").unwrap();

        let first = rewrite(root, "0.1.0-alpha.3").unwrap();
        assert_eq!(first.len(), 1);
        let second = rewrite(root, "0.1.0-alpha.3").unwrap();
        assert!(second.is_empty(), "second rewrite should be a no-op");
    }
}
