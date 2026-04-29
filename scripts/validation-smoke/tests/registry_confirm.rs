use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Output},
    thread,
};

use sha3::{Digest, Keccak256};
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

fn write_provenance(path: &std::path::Path, code_hash: &str) {
    fs::write(
        path,
        format!(
            "version: 1\n\
             provenance:\n\
             - contract_id: Settlement\n\
             \x20 chain_id: 1\n\
             \x20 env: prod\n\
             \x20 address: '0x1111111111111111111111111111111111111111'\n\
             \x20 live_confirmation:\n\
             \x20\x20 kind: code_hash\n\
             \x20\x20 code_hash: '{code_hash}'\n\
             \x20\x20 selector_check:\n\
             \x20\x20\x20 enabled: false\n\
             \x20\x20 rpc_chain_id: 1\n\
             \x20\x20 confirmed_at: '2026-04-28T00:00:00Z'\n\
             \x20\x20 confirmer: test-fixture\n"
        ),
    )
    .unwrap();
}

fn code_hash(bytecode: &[u8]) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(bytecode);
    format!("0x{}", hex::encode(hasher.finalize()))
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
            } else if request.contains("\"eth_call\"") {
                serde_json::json!("0x01")
            } else {
                serde_json::json!(null)
            };
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": result,
            })
            .to_string();
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
fn local_check_tolerates_missing_rpc_and_does_not_mutate_yaml() {
    let temp = tempdir().unwrap();
    let provenance = temp.path().join("deployment-provenance.yaml");
    write_provenance(&provenance, &code_hash(&[0x60, 0x01]));
    let before = fs::read_to_string(&provenance).unwrap();

    let output = command()
        .args([
            "registry-confirm",
            "--mode",
            "local",
            "--check",
            "--chain-ids",
            "1",
            "--provenance-yaml",
            provenance.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    assert_eq!(fs::read_to_string(&provenance).unwrap(), before);
    assert!(output_text(&output).contains("skipped"));
}

#[test]
fn local_write_records_skipped_confirmation_when_rpc_is_missing() {
    let temp = tempdir().unwrap();
    let provenance = temp.path().join("deployment-provenance.yaml");
    write_provenance(&provenance, &code_hash(&[0x60, 0x01]));

    let output = command()
        .args([
            "registry-confirm",
            "--mode",
            "local",
            "--write",
            "--chain-ids",
            "1",
            "--provenance-yaml",
            provenance.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    let updated = fs::read_to_string(&provenance).unwrap();
    assert!(updated.contains("kind: skipped"));
    assert!(updated.contains("missing RPC_1"));
}

#[test]
fn release_check_fails_when_prod_rpc_is_missing() {
    let temp = tempdir().unwrap();
    let provenance = temp.path().join("deployment-provenance.yaml");
    write_provenance(&provenance, &code_hash(&[0x60, 0x01]));

    let output = command()
        .args([
            "registry-confirm",
            "--mode",
            "release",
            "--check",
            "--chain-ids",
            "1",
            "--provenance-yaml",
            provenance.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("missing RPC_1"));
}

#[test]
fn release_check_rejects_zero_code_hash_sentinel_before_rpc() {
    let temp = tempdir().unwrap();
    let provenance = temp.path().join("deployment-provenance.yaml");
    write_provenance(
        &provenance,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    );

    let output = command()
        .args([
            "registry-confirm",
            "--mode",
            "release",
            "--check",
            "--chain-ids",
            "1",
            "--provenance-yaml",
            provenance.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("all-zero code_hash sentinel"));
}

#[test]
fn release_write_refreshes_live_code_hash_from_rpc() {
    let temp = tempdir().unwrap();
    let provenance = temp.path().join("deployment-provenance.yaml");
    write_provenance(
        &provenance,
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );
    let url = start_rpc_server("0x1", "0x6001", 2);
    let expected_hash = code_hash(&[0x60, 0x01]);

    let output = command()
        .env("RPC_1", url)
        .args([
            "registry-confirm",
            "--mode",
            "release",
            "--write",
            "--chain-ids",
            "1",
            "--provenance-yaml",
            provenance.to_str().unwrap(),
            "--confirmer",
            "test-release-write",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    let updated = fs::read_to_string(&provenance).unwrap();
    assert!(updated.contains(&expected_hash));
    assert!(updated.contains("test-release-write"));
}

#[test]
fn release_check_fails_on_committed_yaml_diff_without_writing() {
    let temp = tempdir().unwrap();
    let provenance = temp.path().join("deployment-provenance.yaml");
    write_provenance(
        &provenance,
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );
    let before = fs::read_to_string(&provenance).unwrap();
    let url = start_rpc_server("0x1", "0x6001", 2);

    let output = command()
        .env("RPC_1", url)
        .args([
            "registry-confirm",
            "--mode",
            "release",
            "--check",
            "--chain-ids",
            "1",
            "--provenance-yaml",
            provenance.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("diff Settlement:1:prod"));
    assert_eq!(fs::read_to_string(&provenance).unwrap(), before);
}
