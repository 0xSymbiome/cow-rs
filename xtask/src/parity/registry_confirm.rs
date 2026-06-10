//! Read-only on-chain presence probe for the deployment registry.
//!
//! For every `(contract_id, chain_id, env)` deployment the SDK's
//! `cow_sdk_contracts::Registry` resolves, this confirms — against a configured
//! RPC — that the chain id matches and that `eth_getCode` returns non-empty
//! bytecode at the resolved address. It never mutates any file: trust rests on the upstream commit pinned
//! in `parity/source-lock.yaml` and the deterministic CREATE2 address, with this
//! probe adding a live check that the claimed deployment actually exists
//! on-chain. Committed per-row code hashes are intentionally not used (see
//! ADR 0032).

use std::{collections::BTreeSet, env, time::Duration};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use cow_sdk_contracts::{ContractId, DeploymentChainId, DeploymentEnv, Registry};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
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

#[derive(Debug, clap::Args)]
pub struct RegistryConfirmArgs {
    /// Missing-RPC policy. `release` fails closed on a missing production-chain
    /// RPC; `local` skips chains whose RPC is not configured.
    #[arg(long, value_enum)]
    pub mode: RegistryMode,
    /// Comma-separated chain ids to probe.
    #[arg(long, value_delimiter = ',', required = true)]
    pub chain_ids: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegistryRowKey {
    contract_id: String,
    chain_id: u64,
    env: String,
}

impl RegistryRowKey {
    fn label(&self) -> String {
        format!("{}:{}:{}", self.contract_id, self.chain_id, self.env)
    }
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
    pub selected_chain_ids: Vec<u64>,
    pub confirmed_rows: usize,
    pub skipped_rows: Vec<RegistryConfirmationSkip>,
    pub failures: Vec<RegistryConfirmationFailure>,
}

impl RegistryConfirmReport {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        i32::from(!self.failures.is_empty())
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "registry-confirm {}: {} present, {} skipped, {} failure(s)",
            self.mode.as_str(),
            self.confirmed_rows,
            self.skipped_rows.len(),
            self.failures.len()
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
        lines.join("\n")
    }
}

enum Presence {
    Confirmed,
    Skipped(String),
}

/// The CREATE2 singletons the registry resolves: settlement, vault-relayer,
/// and eth-flow each carry a production and a staging deployment.
const PROBES: [(ContractId, DeploymentEnv); 6] = [
    (ContractId::Settlement, DeploymentEnv::Prod),
    (ContractId::Settlement, DeploymentEnv::Staging),
    (ContractId::VaultRelayer, DeploymentEnv::Prod),
    (ContractId::VaultRelayer, DeploymentEnv::Staging),
    (ContractId::EthFlow, DeploymentEnv::Prod),
    (ContractId::EthFlow, DeploymentEnv::Staging),
];

pub fn run(args: &RegistryConfirmArgs) -> Result<RegistryConfirmReport> {
    let selected: BTreeSet<u64> = args.chain_ids.iter().copied().collect();
    let selected_chain_ids = selected.iter().copied().collect::<Vec<_>>();

    let registry = Registry::default();
    let client = Client::builder()
        .user_agent("cow-rs-xtask/confirm-deployments")
        .timeout(Duration::from_secs(20))
        .build()
        .context("failed to build registry-confirm HTTP client")?;

    let mut confirmed_rows = 0usize;
    let mut skipped_rows = Vec::new();
    let mut failures = Vec::new();
    let mut matched = 0usize;

    for &chain_id in &selected {
        let Ok(chain) = DeploymentChainId::try_from(chain_id) else {
            continue;
        };
        for (contract_id, env) in PROBES {
            let Some(address) = registry.address(contract_id, chain, env) else {
                continue;
            };
            matched += 1;
            let row = RegistryRowKey {
                contract_id: contract_id.as_str().to_owned(),
                chain_id,
                env: env.as_str().to_owned(),
            };
            match probe_presence(
                &client,
                chain_id,
                env.as_str(),
                &address.to_hex_string(),
                args.mode,
            ) {
                Ok(Presence::Confirmed) => confirmed_rows += 1,
                Ok(Presence::Skipped(reason)) => {
                    skipped_rows.push(RegistryConfirmationSkip { row, reason });
                }
                Err(error) => failures.push(RegistryConfirmationFailure {
                    row,
                    message: error.to_string(),
                }),
            }
        }
    }

    if matched == 0 {
        bail!(
            "no deployment rows matched --chain-ids {}",
            args.chain_ids
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    Ok(RegistryConfirmReport {
        mode: args.mode,
        selected_chain_ids,
        confirmed_rows,
        skipped_rows,
        failures,
    })
}

fn probe_presence(
    client: &Client,
    chain_id: u64,
    env: &str,
    address: &str,
    mode: RegistryMode,
) -> Result<Presence> {
    let env_names = rpc_env_names(chain_id);
    let rpc_url = env_names.iter().find_map(|name| {
        env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| (name.clone(), value))
    });

    let Some((env_name, rpc_url)) = rpc_url else {
        let required = format!("missing {}", env_names.join(" or "));
        if mode == RegistryMode::Release && env == "prod" {
            bail!("{required}");
        }
        return Ok(Presence::Skipped(format!(
            "{required}; skipped in {} mode",
            mode.as_str()
        )));
    };

    let rpc_chain_id = rpc_chain_id(client, &rpc_url)
        .with_context(|| format!("{env_name} eth_chainId request failed"))?;
    if rpc_chain_id != chain_id {
        bail!("{env_name} returned chain id {rpc_chain_id}, expected {chain_id}");
    }

    let code = rpc_get_code(client, &rpc_url, address)
        .with_context(|| format!("{env_name} eth_getCode request failed"))?;
    if code.is_empty() {
        bail!("registry row claims a deployment but eth_getCode is empty at {address}");
    }

    Ok(Presence::Confirmed)
}

fn rpc_env_names(chain_id: u64) -> Vec<String> {
    let mut names = vec![format!("RPC_{chain_id}")];
    let alias = match chain_id {
        1 => Some("RPC_MAINNET"),
        56 => Some("RPC_BNB"),
        100 => Some("RPC_GNOSIS"),
        137 => Some("RPC_POLYGON"),
        232 => Some("RPC_LENS"),
        8453 => Some("RPC_BASE"),
        9745 => Some("RPC_PLASMA"),
        42_161 => Some("RPC_ARBITRUM"),
        43_114 => Some("RPC_AVALANCHE"),
        57_073 => Some("RPC_INK"),
        59_144 => Some("RPC_LINEA"),
        11_155_111 => Some("RPC_SEPOLIA"),
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

#[allow(
    clippy::needless_pass_by_value,
    reason = "params is moved into the request body; the json! macro borrow hides the move from the lint"
)]
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
        .map_err(|error| anyhow::anyhow!(redact_rpc_error(&error.to_string())))
        .with_context(|| format!("{method} HTTP request failed"))?
        .error_for_status()
        .map_err(|error| anyhow::anyhow!(redact_rpc_error(&error.to_string())))
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

/// Replace any `http(s)://...` token in an error string with a placeholder so
/// keyed RPC endpoint URLs (provider API keys live in the path or query) never
/// reach logs.
fn redact_rpc_error(message: &str) -> String {
    message
        .split_whitespace()
        .map(|token| {
            if token.starts_with("http://") || token.starts_with("https://") {
                "<redacted-rpc-url>"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

#[cfg(test)]
mod tests {
    use super::{decode_hex_bytes, parse_hex_u64, redact_rpc_error};

    #[test]
    fn hex_helpers_parse_rpc_shapes() {
        assert_eq!(parse_hex_u64("0x64").unwrap(), 100);
        assert_eq!(decode_hex_bytes("0x6001").unwrap(), vec![0x60, 0x01]);
        assert!(decode_hex_bytes("0x").unwrap().is_empty());
    }

    #[test]
    fn redacts_rpc_urls_from_error_text() {
        let redacted = redact_rpc_error("rpc send failed for https://rpc.example/key123 timeout");
        assert!(!redacted.contains("key123"));
        assert!(redacted.contains("<redacted-rpc-url>"));
    }
}
