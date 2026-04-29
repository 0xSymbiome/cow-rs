use std::{collections::BTreeSet, env, fs, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use clap::{ArgGroup, ValueEnum};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha3::{Digest, Keccak256};

const ZERO_CODE_HASH: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RegistryMode {
    Local,
    Release,
}

impl RegistryMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Release => "release",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryAction {
    Check,
    Write,
}

impl RegistryAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Write => "write",
        }
    }
}

#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").required(true).args(["check", "write"])))]
pub struct RegistryConfirmArgs {
    /// Missing RPC policy to use while confirming selected chains.
    #[arg(long, value_enum)]
    pub mode: RegistryMode,
    /// Recompute live evidence and compare it with the committed YAML.
    #[arg(long, conflicts_with = "write")]
    pub check: bool,
    /// Refresh live_confirmation blocks in the provenance YAML.
    #[arg(long)]
    pub write: bool,
    /// Comma-separated chain ids to confirm.
    #[arg(long, value_delimiter = ',', required = true)]
    pub chain_ids: Vec<u64>,
    /// Deployment provenance manifest to read or update.
    #[arg(long, default_value = "crates/contracts/deployment-provenance.yaml")]
    pub provenance_yaml: PathBuf,
    /// Identifier written into live_confirmation.confirmer during --write.
    #[arg(long, default_value = "validation-smoke/registry-confirm")]
    pub confirmer: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProvenanceManifest {
    version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_at_utc: Option<String>,
    #[serde(default)]
    provenance: Vec<ProvenanceEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProvenanceEntry {
    contract_id: String,
    chain_id: u64,
    env: String,
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    authority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    live_confirmation: Option<LiveConfirmation>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct LiveConfirmation {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    selector_check: Option<SelectorCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rpc_chain_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct SelectorCheck {
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegistryRowKey {
    contract_id: String,
    chain_id: u64,
    env: String,
}

impl RegistryRowKey {
    fn from_entry(entry: &ProvenanceEntry) -> Self {
        Self {
            contract_id: entry.contract_id.clone(),
            chain_id: entry.chain_id,
            env: entry.env.clone(),
        }
    }

    fn label(&self) -> String {
        format!("{}:{}:{}", self.contract_id, self.chain_id, self.env)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegistryConfirmationDiff {
    row: RegistryRowKey,
    expected: Value,
    actual: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegistryConfirmationFailure {
    row: RegistryRowKey,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegistryConfirmationSkip {
    row: RegistryRowKey,
    reason: String,
}

#[derive(Debug, Serialize)]
pub struct RegistryConfirmReport {
    pub mode: RegistryMode,
    pub action: RegistryAction,
    pub provenance_yaml: String,
    pub selected_chain_ids: Vec<u64>,
    pub confirmed_rows: usize,
    pub skipped_rows: Vec<RegistryConfirmationSkip>,
    pub failures: Vec<RegistryConfirmationFailure>,
    pub diffs: Vec<RegistryConfirmationDiff>,
}

impl RegistryConfirmReport {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        if !self.failures.is_empty() {
            return 1;
        }
        if self.mode == RegistryMode::Release
            && self.action == RegistryAction::Check
            && !self.diffs.is_empty()
        {
            return 1;
        }
        0
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "registry-confirm {} --{}: {} confirmed, {} skipped, {} failure(s), {} diff(s)",
            self.mode.as_str(),
            self.action.as_str(),
            self.confirmed_rows,
            self.skipped_rows.len(),
            self.failures.len(),
            self.diffs.len()
        )];
        for skipped in &self.skipped_rows {
            lines.push(format!(
                "  skipped {}: {}",
                skipped.row.label(),
                skipped.reason
            ));
        }
        for failure in &self.failures {
            lines.push(format!(
                "  failed {}: {}",
                failure.row.label(),
                failure.message
            ));
        }
        for diff in &self.diffs {
            lines.push(format!("  diff {}", diff.row.label()));
        }
        lines.join("\n")
    }
}

pub fn run(args: &RegistryConfirmArgs) -> Result<RegistryConfirmReport> {
    let action = action_from_flags(args.check, args.write)?;
    let selected_chain_ids: BTreeSet<u64> = args.chain_ids.iter().copied().collect();
    let selected_chain_ids_vec = selected_chain_ids.iter().copied().collect::<Vec<_>>();
    let raw = fs::read_to_string(&args.provenance_yaml)
        .with_context(|| format!("failed to read {}", args.provenance_yaml.display()))?;
    let mut manifest: ProvenanceManifest = serde_norway::from_str(&raw)
        .with_context(|| format!("failed to parse {}", args.provenance_yaml.display()))?;
    let client = Client::builder()
        .user_agent("cow-rs-validation-smoke/registry-confirm")
        .timeout(Duration::from_secs(20))
        .build()
        .context("failed to build registry-confirm HTTP client")?;

    let mut report = RegistryConfirmReport {
        mode: args.mode,
        action,
        provenance_yaml: args.provenance_yaml.display().to_string(),
        selected_chain_ids: selected_chain_ids_vec,
        confirmed_rows: 0,
        skipped_rows: Vec::new(),
        failures: Vec::new(),
        diffs: Vec::new(),
    };

    let mut matched = 0usize;
    let confirmed_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    for entry in &mut manifest.provenance {
        if !selected_chain_ids.contains(&entry.chain_id) {
            continue;
        }
        matched += 1;
        let row = RegistryRowKey::from_entry(entry);
        if args.mode == RegistryMode::Release
            && action == RegistryAction::Check
            && entry.env == "prod"
            && let Err(error) = validate_committed_release_confirmation(entry)
        {
            report.failures.push(RegistryConfirmationFailure {
                row,
                message: error.to_string(),
            });
            continue;
        }
        match confirm_entry(
            &client,
            entry,
            args.mode,
            action,
            &confirmed_at,
            &args.confirmer,
        ) {
            Ok(EntryConfirmation::Confirmed(next)) => {
                report.confirmed_rows += 1;
                if action == RegistryAction::Check {
                    if entry.live_confirmation.as_ref() != Some(&next) {
                        report.diffs.push(RegistryConfirmationDiff {
                            row,
                            expected: serde_json::to_value(&entry.live_confirmation)
                                .context("failed to render expected live_confirmation")?,
                            actual: serde_json::to_value(&next)
                                .context("failed to render actual live_confirmation")?,
                        });
                    }
                } else {
                    entry.live_confirmation = Some(next);
                }
            }
            Ok(EntryConfirmation::Skipped { next, reason }) => {
                let release_non_prod_skip =
                    args.mode == RegistryMode::Release && entry.env != "prod";
                report.skipped_rows.push(RegistryConfirmationSkip {
                    row: row.clone(),
                    reason,
                });
                if action == RegistryAction::Check {
                    if !release_non_prod_skip && entry.live_confirmation.as_ref() != Some(&next) {
                        report.diffs.push(RegistryConfirmationDiff {
                            row,
                            expected: serde_json::to_value(&entry.live_confirmation)
                                .context("failed to render expected skipped live_confirmation")?,
                            actual: serde_json::to_value(&next)
                                .context("failed to render actual skipped live_confirmation")?,
                        });
                    }
                } else {
                    entry.live_confirmation = Some(next);
                }
            }
            Err(error) => {
                report.failures.push(RegistryConfirmationFailure {
                    row,
                    message: error.to_string(),
                });
            }
        }
    }

    if matched == 0 {
        bail!(
            "no provenance entries matched --chain-ids {}",
            args.chain_ids
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    if action == RegistryAction::Write && report.failures.is_empty() {
        let serialized =
            serde_norway::to_string(&manifest).context("failed to serialize provenance YAML")?;
        fs::write(&args.provenance_yaml, serialized)
            .with_context(|| format!("failed to write {}", args.provenance_yaml.display()))?;
    }

    Ok(report)
}

fn action_from_flags(check: bool, write: bool) -> Result<RegistryAction> {
    match (check, write) {
        (true, false) => Ok(RegistryAction::Check),
        (false, true) => Ok(RegistryAction::Write),
        (false, false) => bail!("registry-confirm requires exactly one of --check or --write"),
        (true, true) => bail!("registry-confirm accepts only one of --check or --write"),
    }
}

fn validate_committed_release_confirmation(entry: &ProvenanceEntry) -> Result<()> {
    let Some(confirmation) = &entry.live_confirmation else {
        bail!("RELEASE-INVALID: production row has no live_confirmation");
    };
    if confirmation.kind != "code_hash" {
        bail!(
            "RELEASE-INVALID: production row has live_confirmation.kind `{}`",
            confirmation.kind
        );
    }

    let Some(code_hash) = confirmation.code_hash.as_deref() else {
        bail!("RELEASE-INVALID: production row has no code_hash");
    };
    if code_hash == ZERO_CODE_HASH {
        bail!("RELEASE-INVALID: production row still has the all-zero code_hash sentinel");
    }
    if !is_32_byte_hex(code_hash) {
        bail!("RELEASE-INVALID: production row has malformed code_hash `{code_hash}`");
    }
    if confirmation
        .confirmed_at
        .as_deref()
        .is_none_or(str::is_empty)
    {
        bail!("RELEASE-INVALID: production row has empty confirmed_at");
    }
    if confirmation.confirmer.as_deref().is_none_or(str::is_empty) {
        bail!("RELEASE-INVALID: production row has empty confirmer");
    }
    Ok(())
}

enum EntryConfirmation {
    Confirmed(LiveConfirmation),
    Skipped {
        next: LiveConfirmation,
        reason: String,
    },
}

fn confirm_entry(
    client: &Client,
    entry: &ProvenanceEntry,
    mode: RegistryMode,
    action: RegistryAction,
    confirmed_at: &str,
    confirmer: &str,
) -> Result<EntryConfirmation> {
    let env_names = rpc_env_names(entry.chain_id);
    let rpc_url = env_names.iter().find_map(|name| {
        env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| (name.clone(), value))
    });

    let Some((env_name, rpc_url)) = rpc_url else {
        let required = format!("missing {}", env_names.join(" or "));
        if mode == RegistryMode::Release && entry.env == "prod" {
            bail!("{required}");
        }
        let reason = format!("{required}; skipped in {} mode", mode.as_str());
        return Ok(EntryConfirmation::Skipped {
            next: skipped_confirmation(entry.live_confirmation.as_ref(), &reason),
            reason,
        });
    };

    let rpc_chain_id = rpc_chain_id(client, &rpc_url)
        .with_context(|| format!("{env_name} eth_chainId request failed"))?;
    if rpc_chain_id != entry.chain_id {
        bail!(
            "{env_name} returned chain id {rpc_chain_id}, expected {}",
            entry.chain_id
        );
    }

    let code = rpc_get_code(client, &rpc_url, &entry.address)
        .with_context(|| format!("{env_name} eth_getCode request failed"))?;
    if code.is_empty() {
        bail!("eth_getCode returned empty bytecode for {}", entry.address);
    }

    let selector_check = run_selector_check(client, &rpc_url, entry)?;
    let mut confirmation = LiveConfirmation {
        kind: "code_hash".to_owned(),
        code_hash: Some(keccak256_hex(&code)),
        selector_check: Some(selector_check),
        rpc_chain_id: Some(rpc_chain_id),
        confirmed_at: Some(confirmed_at.to_owned()),
        confirmer: Some(confirmer.to_owned()),
        reason: None,
    };

    if action == RegistryAction::Check
        && let Some(existing) = &entry.live_confirmation
    {
        confirmation.confirmed_at = existing
            .confirmed_at
            .clone()
            .or_else(|| confirmation.confirmed_at.clone());
        confirmation.confirmer = existing
            .confirmer
            .clone()
            .or_else(|| confirmation.confirmer.clone());
    }

    Ok(EntryConfirmation::Confirmed(confirmation))
}

fn skipped_confirmation(existing: Option<&LiveConfirmation>, reason: &str) -> LiveConfirmation {
    let selector_check = existing.and_then(|confirmation| confirmation.selector_check.clone());
    LiveConfirmation {
        kind: "skipped".to_owned(),
        code_hash: None,
        selector_check,
        rpc_chain_id: None,
        confirmed_at: existing.and_then(|confirmation| confirmation.confirmed_at.clone()),
        confirmer: existing.and_then(|confirmation| confirmation.confirmer.clone()),
        reason: Some(reason.to_owned()),
    }
}

fn rpc_env_names(chain_id: u64) -> Vec<String> {
    let mut names = vec![format!("RPC_{chain_id}")];
    let alias = match chain_id {
        1 => Some("RPC_MAINNET"),
        56 => Some("RPC_BNB"),
        100 => Some("RPC_GNOSIS"),
        137 => Some("RPC_POLYGON"),
        8453 => Some("RPC_BASE"),
        9745 => Some("RPC_PLASMA"),
        42161 => Some("RPC_ARBITRUM"),
        43114 => Some("RPC_AVALANCHE"),
        57073 => Some("RPC_INK"),
        59144 => Some("RPC_LINEA"),
        11155111 => Some("RPC_SEPOLIA"),
        _ => None,
    };
    if let Some(alias) = alias {
        names.push(alias.to_owned());
    }
    names
}

fn rpc_chain_id(client: &Client, rpc_url: &str) -> Result<u64> {
    let value = rpc_request(client, rpc_url, "eth_chainId", json!([]))?;
    let raw = value
        .as_str()
        .context("eth_chainId result must be a hex string")?;
    parse_hex_u64(raw).with_context(|| format!("invalid eth_chainId result `{raw}`"))
}

fn rpc_get_code(client: &Client, rpc_url: &str, address: &str) -> Result<Vec<u8>> {
    let value = rpc_request(client, rpc_url, "eth_getCode", json!([address, "latest"]))?;
    let raw = value
        .as_str()
        .context("eth_getCode result must be a hex string")?;
    decode_hex_bytes(raw).with_context(|| format!("invalid eth_getCode result `{raw}`"))
}

fn run_selector_check(
    client: &Client,
    rpc_url: &str,
    entry: &ProvenanceEntry,
) -> Result<SelectorCheck> {
    let Some(existing) = entry
        .live_confirmation
        .as_ref()
        .and_then(|confirmation| confirmation.selector_check.clone())
    else {
        return Ok(SelectorCheck {
            enabled: false,
            ..SelectorCheck::default()
        });
    };

    if !existing.enabled {
        return Ok(SelectorCheck {
            enabled: false,
            selector: existing.selector,
            ..SelectorCheck::default()
        });
    }

    let Some(selector) = existing.selector.clone() else {
        return Ok(SelectorCheck {
            enabled: true,
            result: Some("failure".to_owned()),
            error: Some("selector_check.enabled=true requires selector".to_owned()),
            ..SelectorCheck::default()
        });
    };

    match rpc_request(
        client,
        rpc_url,
        "eth_call",
        json!([{ "to": entry.address, "data": selector }, "latest"]),
    ) {
        Ok(_) => Ok(SelectorCheck {
            enabled: true,
            selector: Some(selector),
            result: Some("success".to_owned()),
            error: None,
        }),
        Err(error) => Ok(SelectorCheck {
            enabled: true,
            selector: Some(selector),
            result: Some("failure".to_owned()),
            error: Some(error.to_string()),
        }),
    }
}

fn rpc_request(client: &Client, rpc_url: &str, method: &str, params: Value) -> Result<Value> {
    let response: Value = client
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .with_context(|| format!("{method} HTTP request failed"))?
        .error_for_status()
        .with_context(|| format!("{method} HTTP status was not successful"))?
        .json()
        .with_context(|| format!("{method} response was not valid JSON"))?;

    if let Some(error) = response.get("error") {
        bail!("{method} returned JSON-RPC error: {error}");
    }
    response
        .get("result")
        .cloned()
        .with_context(|| format!("{method} response did not contain result"))
}

fn decode_hex_bytes(raw: &str) -> Result<Vec<u8>> {
    let value = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    if value.is_empty() {
        return Ok(Vec::new());
    }
    Ok(hex::decode(value)?)
}

fn parse_hex_u64(raw: &str) -> Result<u64> {
    let value = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    Ok(u64::from_str_radix(value, 16)?)
}

fn is_32_byte_hex(value: &str) -> bool {
    let Some(body) = value.strip_prefix("0x") else {
        return false;
    };
    body.len() == 64 && body.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn keccak256_hex(bytes: &[u8]) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    format!("0x{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{action_from_flags, decode_hex_bytes, keccak256_hex, parse_hex_u64};

    #[test]
    fn action_flags_require_exactly_one_action() {
        assert!(action_from_flags(true, false).is_ok());
        assert!(action_from_flags(false, true).is_ok());
        assert!(action_from_flags(false, false).is_err());
        assert!(action_from_flags(true, true).is_err());
    }

    #[test]
    fn hex_helpers_parse_rpc_shapes() {
        assert_eq!(parse_hex_u64("0x64").unwrap(), 100);
        assert_eq!(decode_hex_bytes("0x6001").unwrap(), vec![0x60, 0x01]);
        assert_eq!(
            keccak256_hex(&[]),
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }
}
