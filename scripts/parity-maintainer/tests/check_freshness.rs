mod common;

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use anyhow::Result;
use tempfile::tempdir;

use common::{RepoSpec, command, output_text, write_source_lock};

#[test]
fn check_freshness_reports_stale_source_lock_without_failing() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_source_lock(
        &root.join("source-lock.yaml"),
        "2025-01-01T00:00:00Z",
        &[RepoSpec {
            id: "services",
            remote: "https://github.com/cowprotocol/services.git".to_string(),
            commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            role: "reference-only",
            producer_paths: vec!["crates/orderbook/openapi.yml"],
        }],
    )?;
    let (api_root, server) = one_response_server(
        "HTTP/1.1 200 OK",
        r#"{"sha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","commit":{"committer":{"date":"2026-04-27T00:00:00Z"}}}"#,
    )?;

    let output = command()
        .current_dir(root)
        .args([
            "check-freshness",
            "--source-lock",
            "source-lock.yaml",
            "--github-api-root",
            &api_root,
            "--now",
            "2026-04-28T00:00:00Z",
        ])
        .output()?;
    server.join().expect("server thread panicked");
    assert!(output.status.success(), "{}", output_text(&output));
    let text = output_text(&output);
    assert!(text.contains("services"));
    assert!(text.contains("stale"));
    assert!(text.contains("older than 90 days"));
    Ok(())
}

#[test]
fn check_freshness_reports_api_errors_as_unknown_and_exits_zero() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_source_lock(
        &root.join("source-lock.yaml"),
        "2026-04-28T00:00:00Z",
        &[RepoSpec {
            id: "services",
            remote: "https://github.com/cowprotocol/services.git".to_string(),
            commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            role: "reference-only",
            producer_paths: vec!["crates/orderbook/openapi.yml"],
        }],
    )?;
    let (api_root, server) =
        one_response_server("HTTP/1.1 500 Internal Server Error", "rate limited")?;

    let output = command()
        .current_dir(root)
        .args([
            "check-freshness",
            "--source-lock",
            "source-lock.yaml",
            "--github-api-root",
            &api_root,
            "--now",
            "2026-04-28T00:00:00Z",
        ])
        .output()?;
    server.join().expect("server thread panicked");
    assert!(output.status.success(), "{}", output_text(&output));
    let text = output_text(&output);
    assert!(text.contains("unknown"));
    assert!(text.contains("informational lane"));
    Ok(())
}

fn one_response_server(
    status: &'static str,
    body: &'static str,
) -> Result<(String, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accepted request");
        let mut buffer = [0_u8; 2048];
        let _ = stream.read(&mut buffer);
        let response = format!(
            "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("wrote mock response");
    });
    Ok((format!("http://{addr}"), handle))
}
