mod common;

use cow_sdk_app_data::{
    AppDataError, CidMode, IpfsConfig, IpfsFetchPolicy, IpfsUploadTransport, SchemaVersion,
    TransportResponse, app_data_hex_to_cid, app_data_hex_to_cid_legacy,
    app_data_hex_to_cid_with_mode, cid_to_app_data_hex, get_app_data_info, get_app_data_schema,
    stringify_deterministic, upload_metadata_doc_to_ipfs_legacy,
};
use serde_json::{Map, Number, Value};

use common::app_data_doc;

const CASE_COUNT: u64 = 128;
const SEARCH_CASE_COUNT: u64 = 512;

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

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
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

fn generated_schema_version(rng: &mut CaseRng) -> String {
    format!(
        "{}.{}.{}",
        rng.next_u32() % 1_000,
        rng.next_u32() % 1_000,
        rng.next_u32() % 1_000
    )
}

fn generated_json_value(rng: &mut CaseRng, depth: usize) -> Value {
    if depth == 0 {
        return match rng.next_u64() % 5 {
            0 => Value::Null,
            1 => Value::Bool(rng.next_bool()),
            2 => Value::Number(Number::from(rng.next_u32() % 10_000)),
            3 => Value::String(format!("value-{}", rng.next_u32())),
            _ => Value::String(format!("0x{}", hex::encode(rng.fill::<4>()))),
        };
    }

    match rng.next_u64() % 4 {
        0 => generated_json_value(rng, 0),
        1 => {
            let len = (rng.next_u64() % 4) as usize;
            Value::Array(
                (0..len)
                    .map(|_| generated_json_value(rng, depth.saturating_sub(1)))
                    .collect(),
            )
        }
        _ => {
            let len = 1 + (rng.next_u64() % 4) as usize;
            let mut object = Map::new();
            for index in 0..len {
                object.insert(
                    format!("key-{}-{}", index, rng.next_u32()),
                    generated_json_value(rng, depth.saturating_sub(1)),
                );
            }
            Value::Object(object)
        }
    }
}

fn manual_canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).unwrap(),
        Value::Array(array) => format!(
            "[{}]",
            array
                .iter()
                .map(manual_canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "{{{}}}",
                entries
                    .into_iter()
                    .map(|(key, item)| {
                        format!(
                            "{}:{}",
                            serde_json::to_string(key).unwrap(),
                            manual_canonical_json(item)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

fn generated_valid_document(rng: &mut CaseRng) -> Value {
    let mut document = Map::new();
    if rng.next_bool() {
        document.insert("metadata".to_owned(), Value::Object(Map::new()));
        document.insert("version".to_owned(), Value::String("0.7.0".to_owned()));
        document.insert("appCode".to_owned(), Value::String("CoW Swap".to_owned()));
    } else {
        document.insert("appCode".to_owned(), Value::String("CoW Swap".to_owned()));
        document.insert("version".to_owned(), Value::String("0.7.0".to_owned()));
        document.insert("metadata".to_owned(), Value::Object(Map::new()));
    }

    if rng.next_bool() {
        document.insert(
            "environment".to_owned(),
            Value::String(format!("env-{}", rng.next_u32() % 16)),
        );
    }

    Value::Object(document)
}

fn reordered_document(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.reverse();
            let mut reordered = Map::new();
            for (key, item) in entries {
                reordered.insert(key.clone(), reordered_document(item));
            }
            Value::Object(reordered)
        }
        Value::Array(array) => Value::Array(array.iter().map(reordered_document).collect()),
        other => other.clone(),
    }
}

fn generated_search_profile_json_value(rng: &mut CaseRng, depth: usize) -> Value {
    if depth == 0 {
        return match rng.next_u64() % 6 {
            0 => Value::Null,
            1 => Value::Bool(rng.next_bool()),
            2 => Value::Number(Number::from(rng.next_u32())),
            3 => Value::String(format!("search-{}", rng.next_u32())),
            4 => Value::String(format!("0x{}", hex::encode(rng.fill::<8>()))),
            _ => Value::String("boundary".repeat(1 + (rng.next_u32() % 3) as usize)),
        };
    }

    match rng.next_u64() % 5 {
        0 => generated_search_profile_json_value(rng, 0),
        1 | 2 => {
            let len = 1 + (rng.next_u64() % 6) as usize;
            Value::Array(
                (0..len)
                    .map(|_| generated_search_profile_json_value(rng, depth.saturating_sub(1)))
                    .collect(),
            )
        }
        _ => {
            let len = 1 + (rng.next_u64() % 6) as usize;
            let mut object = Map::new();
            for index in 0..len {
                object.insert(
                    format!("search-key-{}-{}", index, rng.next_u32()),
                    generated_search_profile_json_value(rng, depth.saturating_sub(1)),
                );
            }
            Value::Object(object)
        }
    }
}

fn generated_search_profile_schema_version(case: u64, rng: &mut CaseRng) -> String {
    match case % 4 {
        0 => format!("0.0.{}", case % 1_000),
        1 => format!("{}.0.0", 1 + (rng.next_u32() % 1_000_000)),
        2 => format!(
            "{}.{}.{}",
            rng.next_u32() % 1_000,
            rng.next_u32() % 10_000,
            rng.next_u32() % 10_000
        ),
        _ => format!(
            "{}.{}.{}",
            10_000 + (case % 10_000),
            rng.next_u32() % 100,
            rng.next_u32() % 100
        ),
    }
}

fn invalid_search_profile_schema_version(case: u64, rng: &mut CaseRng) -> String {
    match case % 8 {
        0 => format!("{}.{}", rng.next_u32() % 100, rng.next_u32() % 100),
        1 => format!(
            "{}.{}.{}.{}",
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            rng.next_u32() % 100
        ),
        2 => format!(
            "v{}.{}.{}",
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            rng.next_u32() % 100
        ),
        3 => format!(
            "{}.{}.-{}",
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            1 + (rng.next_u32() % 100)
        ),
        4 => format!(
            " {}.{}.{}",
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            rng.next_u32() % 100
        ),
        5 => format!(
            "{}.{}.{} ",
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            rng.next_u32() % 100
        ),
        6 => format!(
            "{}.{}.{}x",
            rng.next_u32() % 100,
            rng.next_u32() % 100,
            rng.next_u32() % 100
        ),
        _ => format!("alpha.{}.{}", rng.next_u32() % 100, rng.next_u32() % 100),
    }
}

#[test]
fn cid_roundtrips_hold_for_latest_and_legacy_modes() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xA770_0001);
        let app_data_hex = generated_app_data_hex(&mut rng);

        let latest = app_data_hex_to_cid(&app_data_hex).unwrap();
        let legacy = app_data_hex_to_cid_legacy(&app_data_hex).unwrap();

        assert_eq!(
            cid_to_app_data_hex(&latest).unwrap(),
            app_data_hex,
            "seed {seed}"
        );
        assert_eq!(
            cid_to_app_data_hex(&legacy).unwrap(),
            app_data_hex,
            "seed {seed}"
        );
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

#[test]
fn deterministic_stringify_is_stable_for_generated_nested_documents() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xA770_0006);
        let document = generated_json_value(&mut rng, 2);
        let rendered = stringify_deterministic(&document).unwrap();

        assert_eq!(rendered, manual_canonical_json(&document), "seed {seed}");
        assert_eq!(
            serde_json::from_str::<Value>(&rendered).unwrap(),
            document,
            "seed {seed}"
        );
        assert_eq!(
            stringify_deterministic(&serde_json::from_str::<Value>(&rendered).unwrap()).unwrap(),
            rendered,
            "seed {seed}"
        );
    }
}

#[test]
fn schema_versions_roundtrip_for_generated_triplets() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xA770_0007);
        let version = generated_schema_version(&mut rng);
        let schema = SchemaVersion::new(version.clone()).unwrap();

        assert_eq!(schema.as_str(), version, "seed {seed}");
        assert_eq!(schema.to_string(), version, "seed {seed}");
        assert_eq!(
            version.parse::<SchemaVersion>().unwrap(),
            schema,
            "seed {seed}"
        );
    }
}

#[test]
fn document_sources_canonicalize_equivalent_top_level_permutations() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xA770_0008);
        let document = generated_valid_document(&mut rng);
        let reordered = reordered_document(&document);

        assert_eq!(
            get_app_data_info(document).unwrap(),
            get_app_data_info(reordered).unwrap(),
            "seed {seed}"
        );
    }
}

#[test]
fn canonicalization_narrow_search_profile_preserves_equivalent_nested_documents() {
    for case in 0..SEARCH_CASE_COUNT {
        let mut rng = CaseRng::new(case ^ 0xA770_0101);
        let document = generated_search_profile_json_value(&mut rng, 3);
        let reordered = reordered_document(&document);
        let rendered = stringify_deterministic(&document).unwrap();

        assert_eq!(rendered, manual_canonical_json(&document), "case {case}");
        assert_eq!(
            rendered,
            stringify_deterministic(&reordered).unwrap(),
            "case {case}"
        );
        assert_eq!(
            serde_json::from_str::<Value>(&rendered).unwrap(),
            document,
            "case {case}"
        );
    }
}

#[test]
fn schema_parsing_narrow_search_profile_roundtrips_valid_triplets_and_rejects_invalid_forms() {
    for case in 0..SEARCH_CASE_COUNT {
        let mut rng = CaseRng::new(case ^ 0xA770_0102);
        let valid = generated_search_profile_schema_version(case, &mut rng);
        let parsed = SchemaVersion::new(valid.clone()).unwrap();
        let invalid = invalid_search_profile_schema_version(case, &mut rng);

        assert_eq!(parsed.as_str(), valid, "case {case}");
        assert_eq!(parsed.to_string(), valid, "case {case}");
        assert_eq!(
            valid.parse::<SchemaVersion>().unwrap(),
            parsed,
            "case {case}"
        );
        assert!(
            SchemaVersion::new(invalid.clone()).is_err(),
            "case {case}: {invalid}"
        );
        assert!(
            invalid.parse::<SchemaVersion>().is_err(),
            "case {case}: {invalid}"
        );
    }
}
