use std::{
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Output},
    thread,
};

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

/// The probe sources its rows from `cow_sdk_contracts::Registry`, so no manifest
/// path is supplied; chain 1 resolves the settlement, vault-relayer, and
/// eth-flow (prod + staging) singletons.
fn args(mode: &str) -> Vec<String> {
    vec![
        "registry-confirm".to_owned(),
        "--mode".to_owned(),
        mode.to_owned(),
        "--chain-ids".to_owned(),
        "1".to_owned(),
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
    let output = command().args(args("local")).output().unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("skipped"));
}

#[test]
fn release_fails_on_missing_prod_rpc() {
    let output = command().args(args("release")).output().unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("missing RPC_1"));
}

#[test]
fn confirms_present_bytecode() {
    // Four rows per chain (settlement, vault-relayer, eth-flow prod + staging),
    // each probed with an eth_chainId + eth_getCode pair.
    let url = start_rpc_server("0x1", "0x6001", 8);

    let output = command()
        .env("RPC_1", url)
        .args(args("release"))
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("present"));
}

#[test]
fn fails_when_bytecode_is_empty() {
    let url = start_rpc_server("0x1", "0x", 8);

    let output = command()
        .env("RPC_1", url)
        .args(args("release"))
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("eth_getCode is empty"));
}

#[test]
fn fails_on_chain_id_mismatch() {
    let url = start_rpc_server("0x2", "0x6001", 4);

    let output = command()
        .env("RPC_1", url)
        .args(args("release"))
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("returned chain id 2"));
}
