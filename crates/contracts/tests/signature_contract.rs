mod common;

use std::{cell::RefCell, fmt, rc::Rc, sync::Mutex};

use alloy_sol_types::SolCall;
use cow_sdk_contracts::{
    ContractsError, Eip1271Cache, Eip1271SignatureData, Eip1271VerificationRequest, IERC1271,
    NoopEip1271Cache, RecoverableSignature, Signature, SigningScheme,
    decode_eip1271_signature_data, decode_signing_scheme, encode_eip1271_signature_data,
    verify_eip1271_signature, verify_eip1271_signature_cached,
};
use cow_sdk_core::{
    Address, Amount, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider, Signer,
    SigningProvider, TransactionBroadcast, TransactionReceipt, TransactionRequest,
};
use sha3::{Digest, Keccak256};

/// Recorded `(verifier, digest, signature_hash)` probe identity.
type CacheWrite = (Address, [u8; 32], [u8; 32]);

#[derive(Default)]
struct RecordingCache {
    hit: Mutex<bool>,
    writes: Mutex<Vec<CacheWrite>>,
}

impl RecordingCache {
    const fn with_valid_hit() -> Self {
        Self {
            hit: Mutex::new(true),
            writes: Mutex::new(Vec::new()),
        }
    }

    fn writes(&self) -> Vec<CacheWrite> {
        self.writes.lock().unwrap().clone()
    }
}

impl Eip1271Cache for RecordingCache {
    fn contains_valid(
        &self,
        _verifier: Address,
        _digest: [u8; 32],
        _signature_hash: [u8; 32],
    ) -> bool {
        *self.hit.lock().unwrap()
    }

    fn record_valid(&self, verifier: Address, digest: [u8; 32], signature_hash: [u8; 32]) {
        self.writes
            .lock()
            .unwrap()
            .push((verifier, digest, signature_hash));
    }
}

use common::MockProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AsyncMockProviderError(String);

impl fmt::Display for AsyncMockProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Default)]
struct DummySigner;

impl Signer for DummySigner {
    type Error = AsyncMockProviderError;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(Address::new("0x1111111111111111111111111111111111111111").unwrap())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err(AsyncMockProviderError("not used".to_owned()))
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err(AsyncMockProviderError("not used".to_owned()))
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &cow_sdk_core::TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Err(AsyncMockProviderError("not used".to_owned()))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(
            Hash32::new(format!("0x{}", "aa".repeat(32))).unwrap(),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

#[derive(Clone)]
struct AsyncMockProvider {
    calls: Rc<RefCell<Vec<ContractCall>>>,
    response: Rc<RefCell<String>>,
    response_error: Rc<RefCell<Option<String>>>,
    code: Rc<RefCell<Option<HexData>>>,
    code_error: Rc<RefCell<Option<String>>>,
}

impl AsyncMockProvider {
    fn new() -> Self {
        Self {
            calls: Rc::new(RefCell::new(Vec::new())),
            response: Rc::new(RefCell::new("null".to_owned())),
            response_error: Rc::new(RefCell::new(None)),
            code: Rc::new(RefCell::new(None)),
            code_error: Rc::new(RefCell::new(None)),
        }
    }

    fn set_code(&self, value: Option<&str>) {
        *self.code.borrow_mut() = value.map(|value| HexData::new(value).unwrap());
    }

    fn set_code_error(&self, value: Option<&str>) {
        *self.code_error.borrow_mut() = value.map(str::to_owned);
    }

    fn set_response(&self, value: &str) {
        let mut response = self.response.borrow_mut();
        value.clone_into(&mut response);
    }

    fn set_response_error(&self, value: Option<&str>) {
        *self.response_error.borrow_mut() = value.map(str::to_owned);
    }
}

impl Provider for AsyncMockProvider {
    type Error = AsyncMockProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        if let Some(message) = self.code_error.borrow().clone() {
            return Err(AsyncMockProviderError(message));
        }
        Ok(self.code.borrow().clone())
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &cow_sdk_core::TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        self.calls.borrow_mut().push(request.clone());
        if let Some(message) = self.response_error.borrow().clone() {
            return Err(AsyncMockProviderError(message));
        }
        Ok(self.response.borrow().clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

impl SigningProvider for AsyncMockProvider {
    type Signer = DummySigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(DummySigner)
    }
}

#[test]
fn signing_scheme_codec_pins_wire_discriminants_and_eip1271_magic_value() {
    assert_eq!(decode_signing_scheme(0).unwrap(), SigningScheme::Eip712);
    assert_eq!(decode_signing_scheme(1).unwrap(), SigningScheme::EthSign);
    assert_eq!(decode_signing_scheme(2).unwrap(), SigningScheme::Eip1271);
    assert_eq!(decode_signing_scheme(3).unwrap(), SigningScheme::PreSign);
    assert!(decode_signing_scheme(4).is_err());

    // EIP-1271 success magic value: the `sol!`-emitted selector on
    // `IERC1271::isValidSignatureCall` is the canonical 0x1626ba7e four bytes.
    assert_eq!(
        IERC1271::isValidSignatureCall::SELECTOR,
        [0x16, 0x26, 0xba, 0x7e]
    );
}

#[test]
fn eip1271_signature_payloads_roundtrip_with_variable_lengths() {
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();

    for signature in [
        "0x",
        "0x1234",
        "0x29a674dfc87f8c78fc2bfbcbe8ffdd435091a6a84bc7761db72a45da453d73ac41c5ce28eceb34be73fddc12a5d04af6e736405e41b613aeefeed3db8122420c1b",
    ] {
        let data = Eip1271SignatureData::new(verifier, HexData::new(signature).unwrap());

        let encoded = encode_eip1271_signature_data(&data).unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();
        assert_eq!(
            decoded.verifier.to_hex_string(),
            data.verifier.to_hex_string()
        );
        assert_eq!(decoded.signature, data.signature);
    }

    assert!(decode_eip1271_signature_data("0x1234").is_err());
}

#[test]
fn signature_helpers_preserve_public_contract_surface() {
    let signer = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let ecdsa = Signature::Ecdsa {
        scheme: SigningScheme::Eip712,
        data: RecoverableSignature::parse_hex(
            "0x29A674DFC87F8C78FC2BFBCBE8FFDD435091A6A84BC7761DB72A45DA453D73AC41C5CE28ECEB34BE73FDDC12A5D04AF6E736405E41B613AEEFEED3DB8122420C1B",
        )
        .unwrap()
        .to_hex_string(),
    };
    let pre_sign = Signature::PreSign { owner: signer };
    let eip1271 = Signature::Eip1271 {
        data: Eip1271SignatureData::new(signer, HexData::new("0x1234").unwrap()),
    };

    assert_eq!(ecdsa.scheme(), SigningScheme::Eip712);
    assert_eq!(pre_sign.scheme(), SigningScheme::PreSign);
    assert_eq!(eip1271.scheme(), SigningScheme::Eip1271);
    assert_eq!(ecdsa.declared_address(), None);
    assert_eq!(pre_sign.declared_address(), Some(&signer));
    assert_eq!(eip1271.declared_address(), Some(&signer));
    assert!(SigningScheme::Eip712.is_ecdsa());
    assert!(SigningScheme::EthSign.is_ecdsa());
    assert!(!SigningScheme::Eip1271.is_ecdsa());
}

// The ECDSA happy-path recovery (EIP-712 and EthSign prehash) is proven on the
// underlying `RecoverableSignature::recover` in `recoverable_signature_contract.rs`;
// `Signature::recover_ecdsa_address` only delegates to it for the `Ecdsa` variant,
// so the contract test here pins the variant dispatch that the delegate adds.
#[test]
fn recover_ecdsa_address_rejects_non_ecdsa_variants() {
    let digest = Hash32::new(format!("0x{}", "11".repeat(32))).unwrap();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    let eip1271 = Signature::Eip1271 {
        data: Eip1271SignatureData::new(verifier, HexData::new("0x1234").unwrap()),
    };
    let pre_sign = Signature::PreSign { owner: verifier };

    for signature in [eip1271, pre_sign] {
        let error = signature.recover_ecdsa_address(&digest).unwrap_err();
        assert!(matches!(error, ContractsError::SignatureSchemeNotEcdsa));
    }
}

#[tokio::test]
async fn eip1271_verification_reads_contract_code_and_magic_value() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    provider.set_code(Some("0x6001600055"));
    provider.set_response("\"0x1626ba7e\"");

    verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "11".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
    )
    .await
    .unwrap();

    let call = provider.calls.borrow().last().cloned().unwrap();
    assert_eq!(call.address, verifier);
    assert_eq!(call.method, "isValidSignature");
    assert!(call.abi_json.contains("\"bytes4\""));
    let args: Vec<String> = serde_json::from_str(&call.args_json).unwrap();
    assert_eq!(args[0], format!("0x{}", "11".repeat(32)));
    assert_eq!(args[1], "0x1234");
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_cache_hit_valid_succeeds_without_provider_call() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    let cache = RecordingCache::with_valid_hit();

    verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x66; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect("a cached valid probe must verify without provider I/O");

    assert!(
        provider.calls.borrow().is_empty(),
        "cache hits must not call the verifier contract",
    );
    assert!(
        cache.writes().is_empty(),
        "cache hits must not re-record the probe",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_verification_records_only_valid_outcomes_keyed_by_signature() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    provider.set_code(Some("0x6001600055"));
    let cache = RecordingCache::default();
    // The key folds in the signature: keccak256 of the "0x1234" signature bytes.
    let signature_hash: [u8; 32] = Keccak256::digest([0x12, 0x34]).into();

    provider.set_response("\"0x1626ba7e\"");
    verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x77; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect("valid magic value must verify");
    assert_eq!(cache.writes(), vec![(verifier, [0x77; 32], signature_hash)]);

    provider.set_response("\"0xffffffff\"");
    let mismatch = verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x78; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect_err("wrong magic value must fail closed");
    assert!(matches!(
        mismatch,
        ContractsError::Eip1271MagicValueMismatch {
            expected: [0x16, 0x26, 0xba, 0x7e],
            actual: [0xff, 0xff, 0xff, 0xff],
        }
    ));
    assert_eq!(
        cache.writes(),
        vec![(verifier, [0x77; 32], signature_hash)],
        "a magic-value mismatch must not be recorded (positive-only cache)",
    );

    provider.set_response("{\"unexpected\":true}");
    let before_malformed = cache.writes();
    let malformed = verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x79; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect_err("malformed responses must not be recorded as verifier outcomes");
    assert!(matches!(
        malformed,
        ContractsError::MalformedEip1271Response { .. }
    ));
    assert_eq!(
        cache.writes(),
        before_malformed,
        "non-cacheable verifier errors must not be recorded",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_verification_fails_closed_for_missing_code_and_transport_errors() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    let missing = verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "22".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoopEip1271Cache,
    )
    .await
    .unwrap_err();
    match &missing {
        ContractsError::UnsupportedEip1271Verifier { verifier: got } => {
            assert_eq!(got.to_hex_string(), verifier.to_hex_string());
        }
        other => panic!("expected UnsupportedEip1271Verifier, got {other:?}"),
    }

    provider.set_code(Some("0x6001600055"));
    provider.set_response_error(Some("rpc unavailable"));
    let transport = verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "33".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoopEip1271Cache,
    )
    .await
    .unwrap_err();
    match &transport {
        ContractsError::Eip1271Provider { operation, message } => {
            assert_eq!(*operation, "read_contract");
            assert_eq!(message.as_inner(), "rpc unavailable");
        }
        other => panic!("expected Eip1271Provider, got {other:?}"),
    }

    provider.set_response_error(None);
    provider.set_code_error(Some("code lookup unavailable"));
    let code_error = verify_eip1271_signature_cached(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "44".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoopEip1271Cache,
    )
    .await
    .unwrap_err();
    match &code_error {
        ContractsError::Eip1271Provider { operation, message } => {
            assert_eq!(*operation, "get_code");
            assert_eq!(message.as_inner(), "code lookup unavailable");
        }
        other => panic!("expected Eip1271Provider, got {other:?}"),
    }
}
