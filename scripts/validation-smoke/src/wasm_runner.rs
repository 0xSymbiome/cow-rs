use std::{
    collections::BTreeMap,
    env, fs,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

const DEFAULT_CONFIG_PATH: &str = ".github/config/wasm-test-versions.yaml";
const DEFAULT_FALLBACK_PATH: &str = "scripts/validation-smoke/data/cft-fallback.json";
const CFT_METADATA_URL: &str = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";
const PLATFORM_DOWNLOADS: [(&str, &str); 3] = [
    ("linux", "linux64"),
    ("windows", "win64"),
    ("macos", "mac-arm64"),
];

#[derive(Debug, clap::Args)]
pub struct WasmRunnerSetupArgs {
    /// Path where webdriver.json should be written. Falls back to WEBDRIVER_JSON.
    #[arg(long)]
    pub webdriver_json: Option<PathBuf>,
    /// Pinned Chrome-for-Testing versions config.
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    pub config: PathBuf,
    /// Installation cache directory for downloaded browser archives.
    #[arg(long)]
    pub install_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RefreshSource {
    Online,
    Fallback,
}

impl RefreshSource {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Fallback => "fallback",
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct WasmRunnerRefreshArgs {
    /// Metadata source used to refresh the pinned browser versions file.
    #[arg(long, value_enum, default_value_t = RefreshSource::Online)]
    pub source: RefreshSource,
    /// Offline Chrome-for-Testing metadata snapshot.
    #[arg(long, default_value = DEFAULT_FALLBACK_PATH)]
    pub fallback_path: PathBuf,
    /// Destination wasm-test-versions.yaml path.
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    pub output: PathBuf,
    /// Override for tests that serve Chrome-for-Testing metadata locally.
    #[arg(long, hide = true, default_value = CFT_METADATA_URL)]
    pub metadata_url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WasmTestVersions {
    pub version: u64,
    pub chrome_for_testing: ChromeForTestingPin,
    pub downloads: BTreeMap<String, PlatformDownloads>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChromeForTestingPin {
    pub channel: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub released_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformDownloads {
    pub chrome: DownloadPin,
    pub chromedriver: DownloadPin,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DownloadPin {
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
pub struct WasmRunnerSetupReport {
    pub config: String,
    pub webdriver_json: String,
    pub install_dir: String,
    pub platform: String,
    pub chrome_binary: String,
    pub chromedriver_binary: String,
}

impl WasmRunnerSetupReport {
    #[must_use]
    pub fn render_text(&self) -> String {
        format!(
            "wasm-runner-setup wrote {}\n  platform: {}\n  chrome: {}\n  chromedriver: {}",
            self.webdriver_json, self.platform, self.chrome_binary, self.chromedriver_binary
        )
    }
}

#[derive(Debug, Serialize)]
pub struct WasmRunnerRefreshReport {
    pub source: RefreshSource,
    pub output: String,
    pub version: String,
    pub released_at: String,
    pub platforms: Vec<String>,
}

impl WasmRunnerRefreshReport {
    #[must_use]
    pub fn render_text(&self) -> String {
        format!(
            "wasm-runner-refresh {} wrote {} for Chrome {} ({})",
            self.source.as_str(),
            self.output,
            self.version,
            self.released_at
        )
    }
}

#[derive(Debug, Deserialize)]
struct CftMetadata {
    timestamp: String,
    channels: BTreeMap<String, CftChannel>,
}

#[derive(Debug, Deserialize)]
struct CftChannel {
    channel: String,
    version: String,
    #[serde(default)]
    revision: Option<String>,
    downloads: CftDownloads,
}

#[derive(Debug, Deserialize)]
struct CftDownloads {
    chrome: Vec<CftDownload>,
    chromedriver: Vec<CftDownload>,
}

#[derive(Clone, Debug, Deserialize)]
struct CftDownload {
    platform: String,
    url: String,
    #[serde(default)]
    sha256: Option<String>,
}

pub fn run_setup(args: &WasmRunnerSetupArgs) -> Result<WasmRunnerSetupReport> {
    let webdriver_json = resolve_webdriver_json(args)?;
    let config = load_config(&args.config)?;
    let platform = current_platform_key()?;
    let downloads = config.downloads.get(platform).with_context(|| {
        format!(
            "{} does not contain downloads for {platform}",
            args.config.display()
        )
    })?;
    let install_dir = args.install_dir.clone().unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".cache")
            .join("wasm-runner")
    });
    let install_root = install_dir
        .join(&config.chrome_for_testing.version)
        .join(platform);
    fs::create_dir_all(&install_root)
        .with_context(|| format!("failed to create {}", install_root.display()))?;

    let client = build_client("cow-rs-validation-smoke/wasm-runner-setup")?;
    let chrome_archive =
        download_and_verify(&client, &downloads.chrome, &install_root, "chrome.zip")?;
    let chromedriver_archive = download_and_verify(
        &client,
        &downloads.chromedriver,
        &install_root,
        "chromedriver.zip",
    )?;

    let chrome_extract_dir = install_root.join("chrome");
    let chromedriver_extract_dir = install_root.join("chromedriver");
    extract_zip(&chrome_archive, &chrome_extract_dir)?;
    extract_zip(&chromedriver_archive, &chromedriver_extract_dir)?;

    let chrome_binary =
        find_binary(&chrome_extract_dir, BinaryKind::Chrome).with_context(|| {
            format!(
                "failed to locate Chrome binary under {}",
                chrome_extract_dir.display()
            )
        })?;
    let chromedriver_binary = find_binary(&chromedriver_extract_dir, BinaryKind::ChromeDriver)
        .with_context(|| {
            format!(
                "failed to locate ChromeDriver binary under {}",
                chromedriver_extract_dir.display()
            )
        })?;
    make_executable(&chrome_binary)?;
    make_executable(&chromedriver_binary)?;
    write_webdriver_json(
        &webdriver_json,
        &chrome_binary,
        &chromedriver_binary,
        &config.chrome_for_testing.version,
        platform,
    )?;

    Ok(WasmRunnerSetupReport {
        config: args.config.display().to_string(),
        webdriver_json: webdriver_json.display().to_string(),
        install_dir: install_root.display().to_string(),
        platform: platform.to_owned(),
        chrome_binary: chrome_binary.display().to_string(),
        chromedriver_binary: chromedriver_binary.display().to_string(),
    })
}

pub fn run_refresh(args: &WasmRunnerRefreshArgs) -> Result<WasmRunnerRefreshReport> {
    let client = build_client("cow-rs-validation-smoke/wasm-runner-refresh")?;
    let metadata = match args.source {
        RefreshSource::Online => fetch_cft_metadata(&client, &args.metadata_url)?,
        RefreshSource::Fallback => read_cft_metadata(&args.fallback_path)?,
    };
    let existing = if args.output.exists() {
        Some(load_config(&args.output)?)
    } else {
        None
    };
    let config = pin_from_metadata(&client, &metadata, args.source, existing.as_ref())?;
    let yaml = render_config(&config);
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&args.output, yaml)
        .with_context(|| format!("failed to write {}", args.output.display()))?;

    Ok(WasmRunnerRefreshReport {
        source: args.source,
        output: args.output.display().to_string(),
        version: config.chrome_for_testing.version,
        released_at: config.chrome_for_testing.released_at,
        platforms: config.downloads.keys().cloned().collect(),
    })
}

pub fn load_config(path: impl AsRef<Path>) -> Result<WasmTestVersions> {
    let path = path.as_ref();
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_norway::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn resolve_webdriver_json(args: &WasmRunnerSetupArgs) -> Result<PathBuf> {
    if let Some(path) = &args.webdriver_json {
        return Ok(path.clone());
    }
    if let Some(path) = env::var_os("WEBDRIVER_JSON")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(path);
    }
    bail!("wasm-runner-setup requires --webdriver-json or WEBDRIVER_JSON")
}

fn fetch_cft_metadata(client: &Client, metadata_url: &str) -> Result<CftMetadata> {
    client
        .get(metadata_url)
        .send()
        .with_context(|| format!("failed to fetch {metadata_url}"))?
        .error_for_status()
        .with_context(|| format!("{metadata_url} returned an unsuccessful status"))?
        .json()
        .with_context(|| format!("failed to parse {metadata_url}"))
}

fn read_cft_metadata(path: &Path) -> Result<CftMetadata> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn pin_from_metadata(
    client: &Client,
    metadata: &CftMetadata,
    source: RefreshSource,
    existing: Option<&WasmTestVersions>,
) -> Result<WasmTestVersions> {
    let stable = metadata
        .channels
        .get("Stable")
        .context("Chrome-for-Testing metadata does not contain Stable channel")?;
    let existing_hashes = existing_hashes_by_url(existing);
    let mut downloads = BTreeMap::new();
    for (logical, cft_platform) in PLATFORM_DOWNLOADS {
        let chrome = find_cft_download(&stable.downloads.chrome, cft_platform, "chrome")?;
        let chromedriver =
            find_cft_download(&stable.downloads.chromedriver, cft_platform, "chromedriver")?;
        downloads.insert(
            logical.to_owned(),
            PlatformDownloads {
                chrome: DownloadPin {
                    url: chrome.url.clone(),
                    sha256: resolve_sha256(client, source, chrome, &existing_hashes)?,
                },
                chromedriver: DownloadPin {
                    url: chromedriver.url.clone(),
                    sha256: resolve_sha256(client, source, chromedriver, &existing_hashes)?,
                },
            },
        );
    }

    Ok(WasmTestVersions {
        version: 1,
        chrome_for_testing: ChromeForTestingPin {
            channel: stable.channel.clone(),
            version: stable.version.clone(),
            revision: stable.revision.clone(),
            released_at: metadata.timestamp.clone(),
        },
        downloads,
    })
}

fn find_cft_download<'a>(
    downloads: &'a [CftDownload],
    platform: &str,
    product: &str,
) -> Result<&'a CftDownload> {
    downloads
        .iter()
        .find(|download| download.platform == platform)
        .with_context(|| format!("Stable metadata is missing {product} download for {platform}"))
}

fn existing_hashes_by_url(existing: Option<&WasmTestVersions>) -> BTreeMap<String, String> {
    let mut hashes = BTreeMap::new();
    if let Some(existing) = existing {
        for downloads in existing.downloads.values() {
            hashes.insert(
                downloads.chrome.url.clone(),
                downloads.chrome.sha256.clone(),
            );
            hashes.insert(
                downloads.chromedriver.url.clone(),
                downloads.chromedriver.sha256.clone(),
            );
        }
    }
    hashes
}

fn resolve_sha256(
    client: &Client,
    source: RefreshSource,
    download: &CftDownload,
    existing_hashes: &BTreeMap<String, String>,
) -> Result<String> {
    if let Some(sha256) = &download.sha256 {
        return normalize_sha256(sha256);
    }
    if let Some(sha256) = existing_hashes.get(&download.url) {
        return normalize_sha256(sha256);
    }
    if source == RefreshSource::Fallback {
        bail!(
            "fallback metadata entry for {} lacks sha256; offline refresh cannot download archives",
            download.url
        );
    }
    download_sha256(client, &download.url)
}

fn normalize_sha256(raw: &str) -> Result<String> {
    let value = raw.trim().to_ascii_lowercase();
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("sha256 must be 64 lowercase or uppercase hex characters");
    }
    Ok(value)
}

fn download_sha256(client: &Client, url: &str) -> Result<String> {
    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("failed to download {url} for sha256"))?
        .error_for_status()
        .with_context(|| format!("{url} returned an unsuccessful status"))?;
    let mut hasher = Sha256::new();
    io::copy(&mut response, &mut hasher).with_context(|| format!("failed to hash {url}"))?;
    Ok(hex::encode(hasher.finalize()))
}

fn download_and_verify(
    client: &Client,
    pin: &DownloadPin,
    install_root: &Path,
    file_name: &str,
) -> Result<PathBuf> {
    let archive_path = install_root.join(file_name);
    if archive_path.exists() && sha256_file(&archive_path)? == pin.sha256 {
        return Ok(archive_path);
    }

    let mut response = client
        .get(&pin.url)
        .send()
        .with_context(|| format!("failed to download {}", pin.url))?
        .error_for_status()
        .with_context(|| format!("{} returned an unsuccessful status", pin.url))?;
    let temp_path = archive_path.with_extension("download");
    let mut file = File::create(&temp_path)
        .with_context(|| format!("failed to create {}", temp_path.display()))?;
    io::copy(&mut response, &mut file)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    drop(file);

    let actual = sha256_file(&temp_path)?;
    if actual != pin.sha256 {
        bail!(
            "sha256 mismatch for {}: expected {}, got {}",
            pin.url,
            pin.sha256,
            actual
        );
    }
    fs::rename(&temp_path, &archive_path).with_context(|| {
        format!(
            "failed to move {} to {}",
            temp_path.display(),
            archive_path.display()
        )
    })?;
    Ok(archive_path)
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn extract_zip(archive_path: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;
    let file = File::open(archive_path)
        .with_context(|| format!("failed to open {}", archive_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("failed to read zip {}", archive_path.display()))?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).with_context(|| {
            format!(
                "failed to open zip entry {index} from {}",
                archive_path.display()
            )
        })?;
        let enclosed = entry
            .enclosed_name()
            .with_context(|| format!("zip entry {} has an unsafe path", entry.name()))?;
        let output_path = destination.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&output_path)
                .with_context(|| format!("failed to create {}", output_path.display()))?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let mut output = File::create(&output_path)
            .with_context(|| format!("failed to create {}", output_path.display()))?;
        io::copy(&mut entry, &mut output)
            .with_context(|| format!("failed to extract {}", output_path.display()))?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum BinaryKind {
    Chrome,
    ChromeDriver,
}

fn find_binary(root: &Path, kind: BinaryKind) -> Result<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in
            fs::read_dir(&path).with_context(|| format!("failed to read {}", path.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read entry in {}", path.display()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if is_candidate_binary(&path, kind) {
                return Ok(path);
            }
        }
    }
    bail!("binary not found")
}

fn is_candidate_binary(path: &Path, kind: BinaryKind) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    match kind {
        BinaryKind::Chrome => matches!(name, "chrome" | "chrome.exe" | "Google Chrome for Testing"),
        BinaryKind::ChromeDriver => matches!(name, "chromedriver" | "chromedriver.exe"),
    }
}

fn write_webdriver_json(
    path: &Path,
    chrome_binary: &Path,
    chromedriver_binary: &Path,
    version: &str,
    platform: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let value = serde_json::json!({
        "goog:chromeOptions": {
            "binary": chrome_binary,
            "args": [
                "--headless=new",
                "--disable-gpu",
                "--no-sandbox",
                // ChromeDriver 132+ blocks the local wasm-bindgen-test web
                // server (a different loopback port) from talking to the
                // browser unless the runner explicitly allow-lists its
                // origin. The wildcard is appropriate here because the
                // browser only ever runs against an ephemeral
                // wasm-bindgen-test session on localhost.
                "--remote-allow-origins=*"
            ]
        },
        "cow:wasmRunner": {
            "chrome": chrome_binary,
            "chromedriver": chromedriver_binary,
            "version": version,
            "platform": platform
        }
    });
    fs::write(
        path,
        serde_json::to_string_pretty(&value).context("failed to serialize webdriver.json")?,
    )
    .with_context(|| format!("failed to write {}", path.display()))
}

fn render_config(config: &WasmTestVersions) -> String {
    let pinned_on = config
        .chrome_for_testing
        .released_at
        .get(0..10)
        .unwrap_or(&config.chrome_for_testing.released_at);
    let mut yaml = format!(
        "# Pinned on {pinned_on} via `validation-smoke wasm-runner-refresh`.\n\
         # Refresh required at every 0.x.0 cut and any time the pin is older than 90 days at release-readiness.\n\
         version: {}\n\
         chrome_for_testing:\n\
         \x20 channel: {}\n\
         \x20 version: {}\n",
        config.version,
        quote_yaml(&config.chrome_for_testing.channel),
        quote_yaml(&config.chrome_for_testing.version),
    );
    if let Some(revision) = &config.chrome_for_testing.revision {
        yaml.push_str(&format!("  revision: {}\n", quote_yaml(revision)));
    }
    yaml.push_str(&format!(
        "  released_at: {}\n",
        quote_yaml(&config.chrome_for_testing.released_at)
    ));
    yaml.push_str("downloads:\n");
    for (platform, downloads) in &config.downloads {
        yaml.push_str(&format!("  {platform}:\n"));
        yaml.push_str("    chrome:\n");
        yaml.push_str(&format!(
            "      url: {}\n",
            quote_yaml(&downloads.chrome.url)
        ));
        yaml.push_str(&format!(
            "      sha256: {}\n",
            quote_yaml(&downloads.chrome.sha256)
        ));
        yaml.push_str("    chromedriver:\n");
        yaml.push_str(&format!(
            "      url: {}\n",
            quote_yaml(&downloads.chromedriver.url)
        ));
        yaml.push_str(&format!(
            "      sha256: {}\n",
            quote_yaml(&downloads.chromedriver.sha256)
        ));
    }
    yaml
}

fn quote_yaml(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn current_platform_key() -> Result<&'static str> {
    if cfg!(target_os = "linux") {
        return Ok("linux");
    }
    if cfg!(target_os = "windows") {
        return Ok("windows");
    }
    if cfg!(target_os = "macos") {
        return Ok("macos");
    }
    bail!("unsupported host platform for wasm-runner-setup")
}

fn build_client(user_agent: &'static str) -> Result<Client> {
    Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(120))
        .build()
        .context("failed to build wasm-runner HTTP client")
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to set executable bit on {}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DownloadPin, normalize_sha256, render_config};
    use crate::wasm_runner::{ChromeForTestingPin, PlatformDownloads, WasmTestVersions};
    use std::collections::BTreeMap;

    #[test]
    fn sha256_validation_accepts_hex_and_normalizes_case() {
        assert_eq!(
            normalize_sha256("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
                .unwrap(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert!(normalize_sha256("abc").is_err());
    }

    #[test]
    fn render_config_emits_expected_schema() {
        let config = WasmTestVersions {
            version: 1,
            chrome_for_testing: ChromeForTestingPin {
                channel: "Stable".to_owned(),
                version: "148.0.7778.56".to_owned(),
                revision: Some("1610480".to_owned()),
                released_at: "2026-04-28T20:36:36.653Z".to_owned(),
            },
            downloads: BTreeMap::from([(
                "linux".to_owned(),
                PlatformDownloads {
                    chrome: DownloadPin {
                        url: "https://example.test/chrome.zip".to_owned(),
                        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .to_owned(),
                    },
                    chromedriver: DownloadPin {
                        url: "https://example.test/chromedriver.zip".to_owned(),
                        sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                            .to_owned(),
                    },
                },
            )]),
        };
        let yaml = render_config(&config);

        assert!(yaml.contains("chrome_for_testing:"));
        assert!(yaml.contains("released_at: '2026-04-28T20:36:36.653Z'"));
        assert!(yaml.contains("downloads:"));
        assert!(yaml.contains("chromedriver:"));
    }
}
