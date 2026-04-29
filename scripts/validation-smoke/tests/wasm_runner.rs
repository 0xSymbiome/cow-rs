use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read, Write},
    net::TcpListener,
    process::{Command, Output},
    thread,
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use validation_smoke::wasm_runner::load_config;
use zip::{ZipWriter, write::SimpleFileOptions};

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_validation-smoke"));
    command.env_remove("WEBDRIVER_JSON");
    command
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn zip_with_file(path: &str, contents: &[u8]) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        zip.start_file(path, SimpleFileOptions::default()).unwrap();
        zip.write_all(contents).unwrap();
        zip.finish().unwrap();
    }
    cursor.into_inner()
}

fn current_chrome_entry() -> &'static str {
    if cfg!(target_os = "windows") {
        "chrome-win64/chrome.exe"
    } else if cfg!(target_os = "macos") {
        "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
    } else {
        "chrome-linux64/chrome"
    }
}

fn current_chromedriver_entry() -> &'static str {
    if cfg!(target_os = "windows") {
        "chromedriver-win64/chromedriver.exe"
    } else if cfg!(target_os = "macos") {
        "chromedriver-mac-arm64/chromedriver"
    } else {
        "chromedriver-linux64/chromedriver"
    }
}

fn write_runner_config(path: &std::path::Path, base_url: &str, chrome_sha: &str, driver_sha: &str) {
    let chrome_url = format!("{base_url}/chrome.zip");
    let driver_url = format!("{base_url}/chromedriver.zip");
    let mut yaml = String::from(
        "version: 1\nchrome_for_testing:\n  channel: Stable\n  version: '148.0.7778.56'\n  revision: '1610480'\n  released_at: '2026-04-28T20:36:36.653Z'\ndownloads:\n",
    );
    for name in ["linux", "windows", "macos"] {
        yaml.push_str(&format!("  {name}:\n"));
        yaml.push_str("    chrome:\n");
        yaml.push_str(&format!("      url: '{chrome_url}'\n"));
        yaml.push_str(&format!("      sha256: '{chrome_sha}'\n"));
        yaml.push_str("    chromedriver:\n");
        yaml.push_str(&format!("      url: '{driver_url}'\n"));
        yaml.push_str(&format!("      sha256: '{driver_sha}'\n"));
    }
    fs::write(path, yaml).unwrap();
}

fn start_static_server(routes: BTreeMap<String, Vec<u8>>, max_requests: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
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
                if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&buffer);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");
            if let Some(body) = routes.get(path) {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(header.as_bytes()).unwrap();
                stream.write_all(body).unwrap();
            } else {
                stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                    .unwrap();
            }
        }
    });
    url
}

#[test]
fn fallback_refresh_writes_offline_deterministic_config_shape() {
    let temp = tempdir().unwrap();
    let fallback = temp.path().join("cft-fallback.json");
    let output = temp.path().join("wasm-test-versions.yaml");
    fs::write(
        &fallback,
        r#"{
  "timestamp": "2026-04-28T20:36:36.653Z",
  "channels": {
    "Stable": {
      "channel": "Stable",
      "version": "148.0.7778.56",
      "revision": "1610480",
      "downloads": {
        "chrome": [
          {"platform":"linux64","url":"https://example.test/linux/chrome.zip","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
          {"platform":"win64","url":"https://example.test/windows/chrome.zip","sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
          {"platform":"mac-arm64","url":"https://example.test/macos/chrome.zip","sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}
        ],
        "chromedriver": [
          {"platform":"linux64","url":"https://example.test/linux/chromedriver.zip","sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},
          {"platform":"win64","url":"https://example.test/windows/chromedriver.zip","sha256":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"},
          {"platform":"mac-arm64","url":"https://example.test/macos/chromedriver.zip","sha256":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"}
        ]
      }
    }
  }
}"#,
    )
    .unwrap();

    let first = command()
        .args([
            "wasm-runner-refresh",
            "--source",
            "fallback",
            "--fallback-path",
            fallback.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(first.status.success(), "{}", output_text(&first));
    let first_yaml = fs::read_to_string(&output).unwrap();

    let second = command()
        .args([
            "wasm-runner-refresh",
            "--source",
            "fallback",
            "--fallback-path",
            fallback.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(second.status.success(), "{}", output_text(&second));

    assert_eq!(fs::read_to_string(&output).unwrap(), first_yaml);
    let config = load_config(&output).unwrap();
    assert_eq!(config.chrome_for_testing.channel, "Stable");
    assert!(config.downloads.contains_key("linux"));
    assert!(config.downloads.contains_key("windows"));
    assert!(config.downloads.contains_key("macos"));
}

#[test]
fn online_refresh_fetches_metadata_and_hashes_mock_downloads() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("wasm-test-versions.yaml");
    let chrome_zip = zip_with_file(current_chrome_entry(), b"chrome");
    let driver_zip = zip_with_file(current_chromedriver_entry(), b"driver");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let metadata = format!(
        r#"{{
  "timestamp": "2026-04-28T20:36:36.653Z",
  "channels": {{
    "Stable": {{
      "channel": "Stable",
      "version": "148.0.7778.56",
      "revision": "1610480",
      "downloads": {{
        "chrome": [
          {{"platform":"linux64","url":"{base_url}/chrome-linux64.zip"}},
          {{"platform":"win64","url":"{base_url}/chrome-win64.zip"}},
          {{"platform":"mac-arm64","url":"{base_url}/chrome-mac-arm64.zip"}}
        ],
        "chromedriver": [
          {{"platform":"linux64","url":"{base_url}/chromedriver-linux64.zip"}},
          {{"platform":"win64","url":"{base_url}/chromedriver-win64.zip"}},
          {{"platform":"mac-arm64","url":"{base_url}/chromedriver-mac-arm64.zip"}}
        ]
      }}
    }}
  }}
}}"#
    );
    let routes = BTreeMap::from([
        ("/metadata.json".to_owned(), metadata.into_bytes()),
        ("/chrome-linux64.zip".to_owned(), chrome_zip.clone()),
        ("/chrome-win64.zip".to_owned(), chrome_zip.clone()),
        ("/chrome-mac-arm64.zip".to_owned(), chrome_zip),
        ("/chromedriver-linux64.zip".to_owned(), driver_zip.clone()),
        ("/chromedriver-win64.zip".to_owned(), driver_zip.clone()),
        ("/chromedriver-mac-arm64.zip".to_owned(), driver_zip),
    ]);
    thread::spawn(move || {
        for stream in listener.incoming().take(7) {
            let mut stream = stream.unwrap();
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&buffer);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");
            let body = routes.get(path).unwrap();
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(body).unwrap();
        }
    });

    let metadata_url = format!("{base_url}/metadata.json");
    let output_cmd = command()
        .args([
            "wasm-runner-refresh",
            "--source",
            "online",
            "--metadata-url",
            &metadata_url,
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output_cmd.status.success(), "{}", output_text(&output_cmd));
    let config = load_config(&output).unwrap();
    assert_eq!(config.chrome_for_testing.version, "148.0.7778.56");
    assert_eq!(config.downloads["linux"].chrome.sha256.len(), 64);
    assert_eq!(config.downloads["windows"].chromedriver.sha256.len(), 64);
}

#[test]
fn setup_downloads_verifies_extracts_and_writes_webdriver_json() {
    let temp = tempdir().unwrap();
    let chrome_zip = zip_with_file(current_chrome_entry(), b"chrome");
    let driver_zip = zip_with_file(current_chromedriver_entry(), b"driver");
    let chrome_sha = sha256(&chrome_zip);
    let driver_sha = sha256(&driver_zip);
    let routes = BTreeMap::from([
        ("/chrome.zip".to_owned(), chrome_zip),
        ("/chromedriver.zip".to_owned(), driver_zip),
    ]);
    let base_url = start_static_server(routes, 2);
    let config = temp.path().join("wasm-test-versions.yaml");
    let webdriver_json = temp.path().join("webdriver.json");
    let install_dir = temp.path().join("install");
    write_runner_config(&config, &base_url, &chrome_sha, &driver_sha);

    let output = command()
        .args([
            "wasm-runner-setup",
            "--config",
            config.to_str().unwrap(),
            "--install-dir",
            install_dir.to_str().unwrap(),
            "--webdriver-json",
            webdriver_json.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_text(&output));
    let value: Value = serde_json::from_str(&fs::read_to_string(&webdriver_json).unwrap()).unwrap();
    assert!(value["goog:chromeOptions"]["binary"].as_str().is_some());
    assert!(value["cow:wasmRunner"]["chromedriver"].as_str().is_some());
}

#[test]
fn setup_reports_structured_error_when_webdriver_path_is_missing() {
    let output = command()
        .args(["--format", "json", "wasm-runner-setup"])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_text(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("VS10002"));
    assert!(stderr.contains("--webdriver-json or WEBDRIVER_JSON"));
}
