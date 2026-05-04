use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn services_drift_script_generates_stable_markdown_report() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let sandbox = temp_sandbox("cow-rs-services-drift");
    let cow_root = sandbox.join("cow-rs");
    let services_root = sandbox.join("services");
    let summary = sandbox.join("summary.md");

    write_fixture_tree(&cow_root, &services_root);

    let script = bash_path(&root.join("scripts/check-services-drift.sh"));
    let command = format!(
        "{} --upstream {} --cow-rs-root {} --summary-output {}",
        shell_quote(&script),
        shell_quote(&bash_path(&services_root)),
        shell_quote(&bash_path(&cow_root)),
        shell_quote(&bash_path(&summary)),
    );
    let output = Command::new(bash_executable())
        .arg("-lc")
        .arg(command)
        .output()
        .expect("bash must be available for the services drift smoke");

    assert!(
        output.status.success(),
        "services drift smoke must succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let report = fs::read_to_string(&summary)
        .unwrap_or_else(|error| panic!("{} must be written: {error}", summary.display()));

    for required in [
        "# Upstream Services Drift Report",
        "| Input | Value |",
        "## errorType Drift",
        "| Classification | Value | Detail |",
        "| match | all compared errorType tags | both sides agree |",
        "## DTO Field Drift",
        "| DTO | Classification | Field | Type |",
        "| all compared DTOs | match | all compared fields | both sides agree |",
        "## Summary Count",
        "| Metric | Count |",
        "| compared DTO pairs | 7 |",
    ] {
        assert!(
            report.contains(required),
            "services drift report must preserve fragment `{required}`",
        );
    }
}

fn temp_sandbox(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
    fs::create_dir_all(&path)
        .unwrap_or_else(|error| panic!("{} must be created: {error}", path.display()));
    path
}

fn write_fixture_tree(cow_root: &Path, services_root: &Path) {
    write_file(
        &cow_root.join("parity/source-lock.yaml"),
        r"repositories:
- id: services
  remote: https://github.com/cowprotocol/services.git
  commit: fixture-pin
",
    );
    write_file(
        &cow_root.join("crates/orderbook/src/rejection.rs"),
        r"
pub enum OrderbookRejection {
    InvalidAmount,
    Unknown,
}
",
    );
    write_file(
        &cow_root.join("crates/orderbook/src/types.rs"),
        r"
pub struct AppDataObject {
    pub full_app_data: String,
}

pub struct OrderCancellations {
    pub order_uids: Vec<String>,
    pub signature: String,
    pub signing_scheme: String,
}

pub struct OrderCreation {
    pub sell_token: String,
    pub buy_token: String,
    pub fee_amount: String,
    pub signature: String,
}

pub struct OrderQuoteRequest {
    pub sell_token: String,
    pub buy_token: String,
    pub from: String,
    pub price_quality: String,
}
",
    );
    write_file(
        &services_root.join("crates/orderbook/src/api.rs"),
        r#"
pub fn route() {
    error("InvalidAmount");
}
"#,
    );
    write_file(&services_root.join("crates/orderbook/src/api/extra.rs"), "");
    write_file(
        &services_root.join("crates/model/src/request.rs"),
        r"
pub struct AppDataObject {
    pub full_app_data: String,
}

pub struct OrderCancellations {
    pub order_uids: Vec<String>,
    pub signature: String,
    pub signing_scheme: String,
}

pub struct OrderCreation {
    pub sell_token: String,
    pub buy_token: String,
    pub fee_amount: String,
    pub signature: String,
}

pub struct OrderQuoteRequest {
    pub sell_token: String,
    pub buy_token: String,
    pub from: String,
    pub price_quality: String,
}
",
    );
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("{} must be created: {error}", parent.display()));
    }
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("{} must be written: {error}", path.display()));
}

#[cfg(windows)]
fn bash_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        let drive = char::from(bytes[0]).to_ascii_lowercase();
        format!("/{drive}/{}", path[2..].trim_start_matches('/'))
    } else {
        path
    }
}

#[cfg(not(windows))]
fn bash_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(windows)]
fn bash_executable() -> &'static str {
    for candidate in [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files\Git\usr\bin\bash.exe",
    ] {
        if Path::new(candidate).is_file() {
            return candidate;
        }
    }
    "bash"
}

#[cfg(not(windows))]
fn bash_executable() -> &'static str {
    "bash"
}
