use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    process::{Command, Output},
    thread,
};

use tempfile::tempdir;

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_validation-smoke"));
    for name in [
        "RPC_1",
        "RPC_MAINNET",
        "RPC_100",
        "RPC_GNOSIS",
        "RPC_11155111",
        "RPC_SEPOLIA",
    ] {
        command.env_remove(name);
    }
    command
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Minimal registry manifest in the shipped `registry.toml` schema:
/// `schema_version = 2` and one Settlement/1/prod `[[entries]]` row. The
/// presence probe reads only `contract_id`, `chain_id`, `env`, and `address`.
fn write_manifest(path: &Path) {
    fs::write(
        path,
        "schema_version = 2\n\
         \n\
         [[entries]]\n\
         contract_id = \"Settlement\"\n\
         chain_id = 1\n\
         env = \"prod\"\n\
         address = \"0x1111111111111111111111111111111111111111\"\n",
    )
    .unwrap();
}

fn args(manifest: &Path, mode: &str) -> Vec<String> {
    vec![
        "registry-confirm".to_owned(),
        "--mode".to_owned(),
        mode.to_owned(),
        "--chain-ids".to_owned(),
        "1".to_owned(),
        "--registry-toml".to_owned(),
        manifest.to_str().unwrap().to_owned(),
    ]
}

fn start_rpc_server(chain_id: &str, code: &str, max_requests: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let chain_id = chain_id.to_owned();
    let code = code.to_owned();
    thread::spawn(move || {
        for stream in listener.incoming().take(max_requests) {
            let mut stream = stream.unwrap();
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if request_complete(&buffer) {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&buffer);
            let result = if request.contains("\"eth_chainId\"") {
                serde_json::json!(chain_id)
            } else if request.contains("\"eth_getCode\"") {
                serde_json::json!(code)
            } else {
                serde_json::json!(null)
            };
            let body =
                serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": result }).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    url
}

fn request_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    buffer.len() >= header_end + 4 + content_length
}

#[test]
fn local_skips_missing_rpc() {
    let temp = tempdir().unwrap();
    let manifest = temp.path().join("registry.toml");
    write_manifest(&manifest);

    let output = command().args(args(&manifest, "local")).output().unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("skipped"));
}

#[test]
fn release_fails_on_missing_prod_rpc() {
    let temp = tempdir().unwrap();
    let manifest = temp.path().join("registry.toml");
    write_manifest(&manifest);

    let output = command().args(args(&manifest, "release")).output().unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("missing RPC_1"));
}

#[test]
fn confirms_present_bytecode() {
    let temp = tempdir().unwrap();
    let manifest = temp.path().join("registry.toml");
    write_manifest(&manifest);
    let url = start_rpc_server("0x1", "0x6001", 2);

    let output = command()
        .env("RPC_1", url)
        .args(args(&manifest, "release"))
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("1 present"));
}

#[test]
fn fails_when_bytecode_is_empty() {
    let temp = tempdir().unwrap();
    let manifest = temp.path().join("registry.toml");
    write_manifest(&manifest);
    let url = start_rpc_server("0x1", "0x", 2);

    let output = command()
        .env("RPC_1", url)
        .args(args(&manifest, "release"))
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("eth_getCode is empty"));
}

#[test]
fn fails_on_chain_id_mismatch() {
    let temp = tempdir().unwrap();
    let manifest = temp.path().join("registry.toml");
    write_manifest(&manifest);
    let url = start_rpc_server("0x2", "0x6001", 1);

    let output = command()
        .env("RPC_1", url)
        .args(args(&manifest, "release"))
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("returned chain id 2"));
}
