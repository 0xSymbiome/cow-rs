const CORE_TRAITS_SOURCE: &str = include_str!("../src/traits.rs");
const HTTP_TRANSPORT_SOURCE: &str = include_str!("../src/transport/http.rs");

#[test]
fn async_provider_trait_shape_unchanged() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "AsyncProvider"),
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
fn async_signing_provider_trait_shape_unchanged() {
    assert_eq!(
        trait_method_signatures(CORE_TRAITS_SOURCE, "AsyncSigningProvider"),
        ["async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;"],
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
