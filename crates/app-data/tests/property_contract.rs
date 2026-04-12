mod common;

use cow_sdk_app_data::{
    AppDataError, CidMode, IpfsConfig, IpfsFetchPolicy, IpfsUploadTransport, TransportResponse,
    app_data_hex_to_cid, app_data_hex_to_cid_legacy, app_data_hex_to_cid_with_mode,
    cid_to_app_data_hex, get_app_data_schema, upload_metadata_doc_to_ipfs_legacy,
};

use common::app_data_doc;

const CASE_COUNT: u64 = 128;

#[derive(Clone)]
struct CaseRng {
    state: u64,
}

impl CaseRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn fill<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0u8; N];
        for byte in &mut bytes {
            *byte = self.next_u64() as u8;
        }
        bytes
    }
}

struct PanicUploadTransport;

impl IpfsUploadTransport for PanicUploadTransport {
    fn post_json(
        &self,
        _uri: &str,
        _body: &str,
        _headers: &[(String, String)],
    ) -> Result<TransportResponse, AppDataError> {
        panic!("invalid credential inputs must fail before transport is called");
    }
}

fn generated_app_data_hex(rng: &mut CaseRng) -> String {
    format!("0x{}", hex::encode(rng.fill::<32>()))
}

fn invalid_app_data_hex(seed: u64, rng: &mut CaseRng) -> String {
    match seed % 3 {
        0 => hex::encode(rng.fill::<32>()),
        1 => format!("0x{}", hex::encode(rng.fill::<31>())),
        _ => format!("0x{}gg", hex::encode(rng.fill::<31>())),
    }
}

fn invalid_schema_version(seed: u64) -> String {
    match seed % 4 {
        0 => format!("{}.{}", seed, seed + 1),
        1 => format!("{}.{}.{}.{}", seed, seed + 1, seed + 2, seed + 3),
        2 => format!("v{}.{}.{}", seed, seed + 1, seed + 2),
        _ => "not.semver.value".to_owned(),
    }
}

#[test]
fn cid_roundtrips_hold_for_latest_and_legacy_modes() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xA770_0001);
        let app_data_hex = generated_app_data_hex(&mut rng);

        let latest = app_data_hex_to_cid(&app_data_hex).unwrap();
        let legacy = app_data_hex_to_cid_legacy(&app_data_hex).unwrap();

        assert_eq!(cid_to_app_data_hex(&latest).unwrap(), app_data_hex, "seed {seed}");
        assert_eq!(cid_to_app_data_hex(&legacy).unwrap(), app_data_hex, "seed {seed}");
        assert_eq!(
            app_data_hex_to_cid_with_mode(&app_data_hex, CidMode::Latest).unwrap(),
            latest,
            "seed {seed}"
        );
        assert_eq!(
            app_data_hex_to_cid_with_mode(&app_data_hex, CidMode::Legacy).unwrap(),
            legacy,
            "seed {seed}"
        );
    }
}

#[test]
fn invalid_app_data_hex_inputs_fail_closed() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xA770_0002);
        let invalid = invalid_app_data_hex(seed, &mut rng);

        assert_eq!(
            app_data_hex_to_cid(&invalid).unwrap_err(),
            AppDataError::InvalidAppDataHex,
            "seed {seed}"
        );
        assert_eq!(
            app_data_hex_to_cid_legacy(&invalid).unwrap_err(),
            AppDataError::InvalidAppDataHex,
            "seed {seed}"
        );
    }
}

#[test]
fn invalid_schema_versions_fail_closed() {
    for seed in 0..CASE_COUNT {
        let version = invalid_schema_version(seed);
        assert_eq!(
            get_app_data_schema(&version).unwrap_err(),
            AppDataError::InvalidSchemaVersion(version.clone()),
            "seed {seed}"
        );
    }
}

#[test]
fn whitespace_only_fetch_base_uris_fail_closed() {
    for seed in 0..CASE_COUNT {
        let blank = " ".repeat(1 + (seed % 6) as usize);
        assert_eq!(
            IpfsFetchPolicy::new(blank).unwrap_err(),
            AppDataError::Transport("ipfs read base uri must not be empty".to_owned()),
            "seed {seed}"
        );
    }
}

#[test]
fn upload_helpers_require_non_empty_credentials_before_transport() {
    for seed in 0..CASE_COUNT {
        let config = match seed % 4 {
            0 => IpfsConfig {
                pinata_api_key: None,
                pinata_api_secret: Some("secret".to_owned()),
                ..IpfsConfig::default()
            },
            1 => IpfsConfig {
                pinata_api_key: Some(String::new()),
                pinata_api_secret: Some("secret".to_owned()),
                ..IpfsConfig::default()
            },
            2 => IpfsConfig {
                pinata_api_key: Some("key".to_owned()),
                pinata_api_secret: None,
                ..IpfsConfig::default()
            },
            _ => IpfsConfig {
                pinata_api_key: Some("key".to_owned()),
                pinata_api_secret: Some(String::new()),
                ..IpfsConfig::default()
            },
        };

        assert_eq!(
            upload_metadata_doc_to_ipfs_legacy(&app_data_doc(), &PanicUploadTransport, &config)
                .unwrap_err(),
            AppDataError::MissingIpfsCredentials,
            "seed {seed}"
        );
    }
}
