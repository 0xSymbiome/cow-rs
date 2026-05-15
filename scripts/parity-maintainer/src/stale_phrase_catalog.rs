//! Canonical stale-phrase regex catalog.
//!
//! Each entry pairs a stable identifier with a regex pattern that catches a
//! specific editorial regression that the composable and COW Shed strategy
//! pack already corrected. The catalog is editorial-hygiene only: it is run
//! as a local pre-push procedure via the `strategy-doc-lint` cargo alias and
//! is not wired as a required-for-merge CI gate.
//!
//! Patterns match outside an `<!-- audit-trail-stale: ID -->` ...
//! `<!-- /audit-trail-stale -->` quarantine block fail the lint. Inside a
//! quarantine block, the patterns are tolerated so verbatim historical
//! quotes can stay in the source for traceability without re-introducing
//! the regression.
//!
//! The catalog grows as new stale-phrase regressions are detected. Every
//! addition lands with a regression test in
//! `scripts/parity-maintainer/tests/stale_phrase_lint.rs` that asserts the
//! new pattern matches the offending text and does not match a neighboring
//! production phrase.

/// One stale-phrase catalog entry.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StalePattern {
    /// Stable identifier used in failure reports and in audit-trail quarantine
    /// markers.
    pub id: &'static str,
    /// Compiled-at-runtime regular expression.
    pub pattern: &'static str,
}

/// Canonical catalog. Keep this list in sync with the strategy pack's
/// stale-phrase section; every addition or removal lands with a regression
/// test asserting the pattern matches the offending text and does not
/// match production phrases.
pub(crate) const STALE_PHRASE_PATTERNS: &[StalePattern] = &[
    StalePattern {
        id: "WORKSPACE_LENS_FALSE",
        pattern: r"\bWorkspace already supports Lens\b",
    },
    StalePattern {
        id: "COW_SDK_LISTS_BOTH",
        pattern: r"\bcow-sdk'?s? `?SupportedChainId`? enum lists both\b",
    },
    StalePattern {
        id: "LENS_PLASMA_AS_PROD",
        pattern: r"\bLens \+ Plasma included as `?prod`?\b",
    },
    StalePattern {
        id: "CURRENT_11_MINUS_OPTIMISM",
        pattern: r"\bcurrent 11 minus Optimism\b",
    },
    StalePattern {
        id: "COMPOSABLE_LENS_KEYED_10",
        pattern: r"\b88 \(composable, of which 10 are Lens-keyed\b",
    },
    StalePattern {
        id: "STANDARD_PROD_ROWS",
        pattern: r"\bstandard `?prod`? rows\b",
    },
    StalePattern {
        id: "LENS_AUTHORITY_COW_SDK",
        pattern: r"\bLens authority.*cow-sdk canonical\b",
    },
    StalePattern {
        id: "OPTIMISM_NOT_DEPLOYED",
        pattern: r"\bOptimism (has no upstream deployment|is not deployed)\b",
    },
    StalePattern {
        id: "COW_SHED_HOOKS_PATH",
        pattern: r"\bcow-shed/src/COWShedHooks\.sol\b",
    },
    StalePattern {
        id: "DEFAULT_ADD_LENS",
        pattern: r"\bdefault.*add `?SupportedChainId::Lens`?\b",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn catalog_holds_ten_entries() {
        assert_eq!(STALE_PHRASE_PATTERNS.len(), 10);
    }

    #[test]
    fn every_pattern_compiles_cleanly() {
        for entry in STALE_PHRASE_PATTERNS {
            Regex::new(entry.pattern).unwrap_or_else(|error| {
                panic!("pattern {} failed to compile: {error}", entry.id);
            });
        }
    }

    #[test]
    fn every_id_is_unique() {
        let ids: std::collections::BTreeSet<&str> =
            STALE_PHRASE_PATTERNS.iter().map(|entry| entry.id).collect();
        assert_eq!(
            ids.len(),
            STALE_PHRASE_PATTERNS.len(),
            "catalog ids must be unique"
        );
    }

    #[test]
    fn workspace_lens_false_pattern_matches_canonical_regression() {
        let regex = Regex::new(STALE_PHRASE_PATTERNS[0].pattern).unwrap();
        assert!(regex.is_match("Workspace already supports Lens (chain 232)"));
    }

    #[test]
    fn cow_shed_hooks_path_pattern_matches_non_existent_path() {
        let entry = STALE_PHRASE_PATTERNS
            .iter()
            .find(|entry| entry.id == "COW_SHED_HOOKS_PATH")
            .expect("catalog must contain COW_SHED_HOOKS_PATH");
        let regex = Regex::new(entry.pattern).unwrap();
        assert!(regex.is_match("Reference cow-shed/src/COWShedHooks.sol for the digest type."));
    }
}
