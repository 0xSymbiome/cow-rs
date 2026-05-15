//! URL provenance check for `parity/source-lock.yaml`.
//!
//! Rejects credential-shaped URLs (anything matching the `user[:password]@`
//! pattern inside an `https://` link). Pinned source-lock entries should
//! reference public, unauthenticated upstream repositories; a credentialed
//! URL means a maintainer accidentally pasted a token-bearing clone URL,
//! which would leak the token to anyone running `git clone` against the
//! checked-in source lock.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail};

/// Walk every line of the source lock and fail on the first credentialed
/// URL. Returns `Ok(())` on a clean file.
pub(crate) fn run(path: &Path) -> Result<()> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read source lock {}", path.display()))?;
    check_text(&raw)?;
    println!("validated URL provenance");
    Ok(())
}

fn check_text(text: &str) -> Result<()> {
    for line in text.lines() {
        // Strip any `://` so we don't match the URL scheme separator itself.
        let scheme_stripped = line.replace("://", "");
        if line.contains("https://") && scheme_stripped.contains('@') {
            bail!("source lock contains credential-shaped URL: {line}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_url_passes() {
        let text = "  remote: https://github.com/cowprotocol/cow-sdk.git\n";
        assert!(check_text(text).is_ok());
    }

    #[test]
    fn credentialed_url_fails() {
        let text = "  remote: https://user:token@github.com/cowprotocol/cow-sdk.git\n";
        assert!(check_text(text).is_err());
    }

    #[test]
    fn user_only_credential_url_fails() {
        let text = "  remote: https://maintainer@github.com/example/repo.git\n";
        assert!(check_text(text).is_err());
    }
}
