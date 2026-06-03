use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Parser, Subcommand, ValueEnum};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::{Map, Value, json};
use validation_smoke::{registry_confirm, wasm_runner};

const STATUS_PASS: &str = "pass";
const STATUS_FAIL: &str = "fail";
const STATUS_UNAVAILABLE: &str = "unavailable";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum SmokeStatus {
    Pass,
    Fail,
    Unavailable,
}

impl SmokeStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => STATUS_PASS,
            Self::Fail => STATUS_FAIL,
            Self::Unavailable => STATUS_UNAVAILABLE,
        }
    }
}

#[derive(Debug, Serialize)]
struct SmokeResult {
    name: String,
    status: SmokeStatus,
    summary: String,
    details: Map<String, Value>,
}

impl SmokeResult {
    fn new(name: impl Into<String>, status: SmokeStatus, summary: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            summary: summary.into(),
            details: Map::new(),
        }
    }

    fn with_detail(mut self, key: impl Into<String>, value: Value) -> Self {
        self.details.insert(key.into(), value);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Parser)]
#[command(
    about = "Run the optional validation smoke checks for environment-sensitive orderbook, subgraph, and browser-wallet confirmation."
)]
struct Cli {
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    #[command(subcommand)]
    command: SmokeCommand,
}

#[derive(Debug, Subcommand)]
enum SmokeCommand {
    #[command(about = "Run the live orderbook version probe through the native example")]
    OrderbookLive,
    #[command(about = "Run the live subgraph totals probe through the native example")]
    SubgraphLive,
    #[command(about = "Check browser-wallet example readiness for injected-wallet confirmation")]
    BrowserWalletLive {
        #[arg(long)]
        url: Option<String>,
    },
    #[command(about = "Run every smoke surface in sequence")]
    All,
    #[command(about = "Confirm deployment provenance against live chain bytecode")]
    RegistryConfirm(registry_confirm::RegistryConfirmArgs),
    #[command(about = "Install the pinned Chrome-for-Testing WASM browser runner")]
    WasmRunnerSetup(wasm_runner::WasmRunnerSetupArgs),
    #[command(about = "Refresh the pinned Chrome-for-Testing WASM browser versions")]
    WasmRunnerRefresh(wasm_runner::WasmRunnerRefreshArgs),
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root should be resolvable")
}

fn insert_string(map: &mut Map<String, Value>, key: &str, value: impl Into<String>) {
    map.insert(key.to_owned(), Value::String(value.into()));
}

fn insert_i64(map: &mut Map<String, Value>, key: &str, value: i64) {
    map.insert(key.to_owned(), Value::Number(value.into()));
}

fn run_checked_command(
    name: &str,
    command: &[&str],
    env_updates: &BTreeMap<String, String>,
    unavailable_fragments: &[&str],
) -> SmokeResult {
    let mut process = Command::new(command[0]);
    process.args(&command[1..]).current_dir(repo_root());
    process.envs(env_updates);
    let output = match process.output() {
        Ok(output) => output,
        Err(error) => {
            return SmokeResult::new(
                name,
                SmokeStatus::Unavailable,
                format!("{name} is unavailable in the current environment"),
            )
            .with_detail("command", Value::String(command.join(" ")))
            .with_detail("reason", Value::String(error.to_string()));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let transcript = format!("{stdout}\n{stderr}").to_lowercase();

    let mut details = Map::new();
    insert_string(&mut details, "command", command.join(" "));
    insert_i64(
        &mut details,
        "exit_code",
        i64::from(output.status.code().unwrap_or(-1)),
    );
    if !stdout.is_empty() {
        insert_string(&mut details, "stdout", &stdout);
    }
    if !stderr.is_empty() {
        insert_string(&mut details, "stderr", &stderr);
    }

    if output.status.success() {
        if let Ok(report) = serde_json::from_str::<Value>(&stdout) {
            details.insert("report".to_owned(), report);
        }
        return SmokeResult {
            name: name.to_owned(),
            status: SmokeStatus::Pass,
            summary: format!("{name} passed"),
            details,
        };
    }

    if unavailable_fragments
        .iter()
        .any(|fragment| transcript.contains(&fragment.to_lowercase()))
    {
        return SmokeResult {
            name: name.to_owned(),
            status: SmokeStatus::Unavailable,
            summary: format!("{name} unavailable in the current environment"),
            details,
        };
    }

    SmokeResult {
        name: name.to_owned(),
        status: SmokeStatus::Fail,
        summary: format!("{name} failed"),
        details,
    }
}

fn http_probe(
    client: &Client,
    name: &str,
    url: &str,
    required_markers: &[&str],
    manual_steps: &[&str],
) -> SmokeResult {
    let response = match client.get(url).send() {
        Ok(response) => response,
        Err(error) => {
            return SmokeResult::new(
                name,
                SmokeStatus::Unavailable,
                format!("{name} is not currently reachable"),
            )
            .with_detail("url", Value::String(url.to_owned()))
            .with_detail("reason", Value::String(error.to_string()));
        }
    };

    let status = response.status();
    if !status.is_success() {
        return SmokeResult::new(
            name,
            SmokeStatus::Unavailable,
            format!("{name} is not currently reachable"),
        )
        .with_detail("url", Value::String(url.to_owned()))
        .with_detail("http_status", json!(status.as_u16()));
    }

    let body = match response.text() {
        Ok(body) => body,
        Err(error) => {
            return SmokeResult::new(
                name,
                SmokeStatus::Fail,
                format!("{name} failed to read the page body"),
            )
            .with_detail("url", Value::String(url.to_owned()))
            .with_detail("reason", Value::String(error.to_string()));
        }
    };

    let missing: Vec<&str> = required_markers
        .iter()
        .copied()
        .filter(|marker| !body.contains(marker))
        .collect();

    let mut details = Map::new();
    insert_string(&mut details, "url", url);
    details.insert("http_status".to_owned(), json!(status.as_u16()));
    if !manual_steps.is_empty() {
        details.insert(
            "manual_steps".to_owned(),
            Value::Array(
                manual_steps
                    .iter()
                    .map(|step| Value::String((*step).to_owned()))
                    .collect(),
            ),
        );
    }

    if missing.is_empty() {
        return SmokeResult {
            name: name.to_owned(),
            status: SmokeStatus::Pass,
            summary: format!("{name} passed"),
            details,
        };
    }

    details.insert(
        "missing_markers".to_owned(),
        Value::Array(
            missing
                .into_iter()
                .map(|marker| Value::String(marker.to_owned()))
                .collect(),
        ),
    );

    SmokeResult {
        name: name.to_owned(),
        status: SmokeStatus::Fail,
        summary: format!("{name} loaded without the expected page markers"),
        details,
    }
}

fn run_orderbook_live() -> SmokeResult {
    run_checked_command(
        "orderbook-live",
        &[
            "cargo",
            "run",
            "--manifest-path",
            "examples/native/Cargo.toml",
            "--example",
            "orderbook_live_probe",
        ],
        &BTreeMap::new(),
        &[
            "request failed:",
            "timed out",
            "name or service not known",
            "dns error",
            "connection refused",
            "service unavailable",
            "bad gateway",
            "gateway timeout",
            "network is unreachable",
            "no route to host",
        ],
    )
}

fn run_subgraph_live() -> SmokeResult {
    let api_key = env::var("THE_GRAPH_API_KEY")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("COW_SMOKE_THE_GRAPH_API_KEY")
                .ok()
                .filter(|value| !value.is_empty())
        });

    let Some(api_key) = api_key else {
        return SmokeResult::new(
            "subgraph-live",
            SmokeStatus::Unavailable,
            "subgraph-live requires a The Graph API key",
        )
        .with_detail(
            "required_env",
            Value::Array(vec![Value::String(
                "THE_GRAPH_API_KEY or COW_SMOKE_THE_GRAPH_API_KEY".to_owned(),
            )]),
        );
    };

    let mut env_updates = BTreeMap::from([("THE_GRAPH_API_KEY".to_owned(), api_key)]);
    if env::var_os("COW_SUBGRAPH_CHAIN_ID").is_none()
        && let Some(chain_id) = env::var("COW_SMOKE_SUBGRAPH_CHAIN_ID")
            .ok()
            .filter(|value| !value.is_empty())
    {
        env_updates.insert("COW_SUBGRAPH_CHAIN_ID".to_owned(), chain_id);
    }

    run_checked_command(
        "subgraph-live",
        &[
            "cargo",
            "run",
            "--manifest-path",
            "examples/native/Cargo.toml",
            "--example",
            "subgraph_live_query",
        ],
        &env_updates,
        &[
            "required for this live example",
            "request failed",
            "timed out",
            "name or service not known",
            "dns error",
            "connection refused",
            "service unavailable",
            "bad gateway",
            "gateway timeout",
            "network is unreachable",
            "no route to host",
        ],
    )
}

fn run_browser_wallet_live(client: &Client, url: &str) -> SmokeResult {
    http_probe(
        client,
        "browser-wallet-live",
        url,
        &[
            "CoW · Browser-Wallet Trade",
            "Discover wallets",
            "Sign & submit swap",
        ],
        &[
            "Open the served example in a real browser session with the target wallet extension installed.",
            "Run Discover wallets and confirm exactly the intended provider appears (none is auto-selected).",
            "Connect the intended wallet and confirm the session line shows the expected account and chain.",
            "Run Sign message to confirm a personal_sign round-trip on the installed extension.",
            "Run Sign & submit swap only when the selected chain and a funded account are intentionally available; confirm the chain-switch prompt to Sepolia.",
        ],
    )
}

fn render_text(results: &[SmokeResult]) {
    for result in results {
        println!(
            "[{}] {}: {}",
            result.status.as_str().to_uppercase(),
            result.name,
            result.summary
        );
        for (key, value) in &result.details {
            match value {
                Value::Array(_) | Value::Object(_) => {
                    println!("  {key}: {}", serde_json::to_string(value).unwrap())
                }
                Value::String(value) => println!("  {key}: {value}"),
                _ => println!("  {key}: {value}"),
            }
        }
    }
}

fn exit_code_for(results: &[SmokeResult]) -> i32 {
    if results
        .iter()
        .any(|result| result.status == SmokeStatus::Fail)
    {
        return 1;
    }
    if results
        .iter()
        .any(|result| result.status == SmokeStatus::Unavailable)
    {
        return 2;
    }
    0
}

fn emit_command_error(format: OutputFormat, code: &str, message: &str) {
    match format {
        OutputFormat::Text => eprintln!("error {code}: {message}"),
        OutputFormat::Json => eprintln!(
            "{}",
            serde_json::to_string(&json!({
                "level": "error",
                "code": code,
                "message": message,
            }))
            .expect("error diagnostic should serialize")
        ),
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        SmokeCommand::RegistryConfirm(args) => match registry_confirm::run(args) {
            Ok(report) => {
                match cli.format {
                    OutputFormat::Text => println!("{}", report.render_text()),
                    OutputFormat::Json => println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .expect("registry-confirm report should serialize")
                    ),
                }
                std::process::exit(report.exit_code());
            }
            Err(error) => {
                emit_command_error(cli.format, "VS10001", &error.to_string());
                std::process::exit(1);
            }
        },
        SmokeCommand::WasmRunnerSetup(args) => match wasm_runner::run_setup(args) {
            Ok(report) => {
                match cli.format {
                    OutputFormat::Text => println!("{}", report.render_text()),
                    OutputFormat::Json => println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .expect("wasm-runner-setup report should serialize")
                    ),
                }
                std::process::exit(0);
            }
            Err(error) => {
                emit_command_error(cli.format, "VS10002", &error.to_string());
                std::process::exit(1);
            }
        },
        SmokeCommand::WasmRunnerRefresh(args) => match wasm_runner::run_refresh(args) {
            Ok(report) => {
                match cli.format {
                    OutputFormat::Text => println!("{}", report.render_text()),
                    OutputFormat::Json => println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .expect("wasm-runner-refresh report should serialize")
                    ),
                }
                std::process::exit(0);
            }
            Err(error) => {
                emit_command_error(cli.format, "VS10003", &error.to_string());
                std::process::exit(1);
            }
        },
        _ => {}
    }

    let client = Client::builder()
        .user_agent("cow-rs-validation-smoke/1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("smoke HTTP client should build");

    let browser_wallet_url = env::var("COW_SMOKE_BROWSER_WALLET_URL")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());

    let results = match cli.command {
        SmokeCommand::OrderbookLive => vec![run_orderbook_live()],
        SmokeCommand::SubgraphLive => vec![run_subgraph_live()],
        SmokeCommand::BrowserWalletLive { url } => {
            let url = url.unwrap_or(browser_wallet_url);
            vec![run_browser_wallet_live(&client, &url)]
        }
        SmokeCommand::All => vec![
            run_orderbook_live(),
            run_subgraph_live(),
            run_browser_wallet_live(&client, &browser_wallet_url),
        ],
        SmokeCommand::RegistryConfirm(_)
        | SmokeCommand::WasmRunnerSetup(_)
        | SmokeCommand::WasmRunnerRefresh(_) => {
            unreachable!("subcommand handled before smoke flow")
        }
    };

    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&results).expect("smoke JSON output should serialize"),
            );
        }
        OutputFormat::Text => render_text(&results),
    }

    std::process::exit(exit_code_for(&results));
}
