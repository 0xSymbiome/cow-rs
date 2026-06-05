const CORE_TRAITS_SOURCE: &str = concat!(
    include_str!("../src/traits/typed_data.rs"),
    "\n",
    include_str!("../src/traits/transaction.rs"),
    "\n",
    include_str!("../src/traits/contract.rs"),
    "\n",
    include_str!("../src/traits/signer.rs"),
    "\n",
    include_str!("../src/traits/provider.rs"),
    "\n",
    include_str!("../src/traits/log_provider.rs"),
);
const HTTP_TRANSPORT_SOURCE: &str = include_str!("../src/transport/http.rs");

#[test]
fn signer_trait_shape_unchanged() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "Signer"),
        [
            "async fn get_address(&self) -> Result<Address, Self::Error>;",
            "async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;",
            "async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;",
            "async fn sign_typed_data(&self, domain: &TypedDataDomain, fields: &[TypedDataField], value_json: &str) -> Result<String, Self::Error>;",
            "async fn send_transaction(&self, tx: &TransactionRequest) -> Result<TransactionBroadcast, Self::Error>;",
            "async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;",
        ],
    );
}

#[test]
fn narrow_signer_capability_traits_are_stable() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "Owner"),
        ["async fn get_address(&self) -> Result<Address, Self::Error>;"],
    );
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "TypedDataSigner"),
        [
            "async fn sign_typed_data(&self, domain: &TypedDataDomain, fields: &[TypedDataField], value_json: &str) -> Result<String, Self::Error>;"
        ],
    );
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "DigestSigner"),
        ["async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error>;"],
    );
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "Eip1193"),
        [
            "async fn request(&self, method: &str, params: &[String]) -> Result<String, Self::Error>;"
        ],
    );

    let typed_data_body = trait_body(CORE_TRAITS_SOURCE, "TypedDataSigner");
    assert!(
        typed_data_body.contains("async fn sign_typed_data_payload("),
        "TypedDataSigner must expose the explicit typed-data payload helper"
    );
}

#[test]
fn provider_trait_shape_unchanged() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "Provider"),
        [
            "async fn get_chain_id(&self) -> Result<ChainId, Self::Error>;",
            "async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error>;",
            "async fn get_transaction_receipt(&self, transaction_hash: &TransactionHash) -> Result<Option<TransactionReceipt>, Self::Error>;",
            "async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error>;",
            "async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error>;",
            "async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error>;",
            "async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error>;",
            "async fn get_contract(&self, address: &Address, abi_json: &str) -> Result<ContractHandle, Self::Error>;",
        ],
    );
}

#[test]
fn signing_provider_trait_shape_unchanged() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "SigningProvider"),
        ["async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;"],
    );
}

#[test]
fn log_provider_trait_shape() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "LogProvider"),
        ["async fn get_logs(&self, query: &LogQuery) -> Result<Vec<RawLog>, Self::Error>;"],
    );
}

#[test]
fn http_transport_trait_shape_unchanged() {
    assert_eq!(
        trait_method_signatures(HTTP_TRANSPORT_SOURCE, "HttpTransport"),
        [
            "async fn get(&self, path: &str, headers: &[(String, String)], timeout: Option<Duration>) -> Result<String, TransportError>;",
            "async fn post(&self, path: &str, body: &str, headers: &[(String, String)], timeout: Option<Duration>) -> Result<String, TransportError>;",
            "async fn put(&self, path: &str, body: &str, headers: &[(String, String)], timeout: Option<Duration>) -> Result<String, TransportError>;",
            "async fn delete(&self, path: &str, body: &str, headers: &[(String, String)], timeout: Option<Duration>) -> Result<String, TransportError>;",
        ],
    );
}

#[test]
fn transaction_receipt_struct_carries_rich_fields() {
    use cow_sdk_core::{Address, Amount, Hash32, TransactionReceipt, TransactionStatus};

    let transaction_hash = Hash32::new(format!("0x{}", "11".repeat(32))).unwrap();
    let block_hash = Hash32::new(format!("0x{}", "22".repeat(32))).unwrap();
    let from = Address::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let to = Address::new("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();

    let from_parts = TransactionReceipt::from_parts(
        transaction_hash,
        Some(TransactionStatus::Success),
        Some(123),
        Some(block_hash),
        Some(Amount::from(21_000u64)),
        Some(from),
        Some(to),
    );
    let from_builders = TransactionReceipt::new(transaction_hash)
        .with_status(TransactionStatus::Success)
        .with_block_number(123)
        .with_block_hash(block_hash)
        .with_gas_used(Amount::from(21_000u64))
        .with_from(from)
        .with_to(to);

    assert_eq!(from_builders, from_parts);
    assert_eq!(from_parts.status, Some(TransactionStatus::Success));
    assert_eq!(from_parts.block_number, Some(123));
    assert_eq!(from_parts.gas_used, Some(Amount::from(21_000u64)));
}

fn trait_method_signatures(source: &str, trait_name: &str) -> Vec<String> {
    let body = trait_body(source, trait_name);
    let mut signatures = Vec::new();
    let mut current = String::new();
    let mut collecting = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("async fn ") || trimmed.starts_with("fn ") {
            collecting = true;
            current.clear();
        }

        if collecting {
            current.push(' ');
            current.push_str(trimmed);
            if trimmed.ends_with(';') {
                signatures.push(normalize_signature(&current));
                collecting = false;
            }
        }
    }

    signatures
}

fn trait_body<'a>(source: &'a str, trait_name: &str) -> &'a str {
    let needle = format!("trait {trait_name}");
    let trait_start = source
        .find(&needle)
        .unwrap_or_else(|| panic!("trait `{trait_name}` must exist"));
    let open_brace = source[trait_start..].find('{').map_or_else(
        || panic!("trait `{trait_name}` must have a body"),
        |offset| trait_start + offset,
    );
    let mut depth = 0_u32;
    let body_start = open_brace + 1;

    for (offset, ch) in source[open_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start..open_brace + offset];
                }
            }
            _ => {}
        }
    }

    panic!("trait `{trait_name}` body must close");
}

fn normalize_signature(signature: &str) -> String {
    let mut normalized = signature.split_whitespace().collect::<Vec<_>>().join(" ");
    for (from, to) in [
        ("( ", "("),
        (" )", ")"),
        (" ,", ","),
        (",)", ")"),
        ("< ", "<"),
        (" >", ">"),
        (" ;", ";"),
    ] {
        normalized = normalized.replace(from, to);
    }
    normalized
}
